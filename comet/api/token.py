import datetime
import time

import aiohttp
from urllib import parse

from comet.proto.galaxy.protocols import communication_service_pb2

from comet.classes.leaderboards import LeaderboardDefinition, LeaderboardEntry
from comet.classes.achievement import UserAchievement, UserAchievementList
from comet.classes.user_stats import GogUserStat, VALUETYPE_INT, VALUETYPE_FLOAT

LOCAL_TIMEZONE = datetime.datetime.utcnow().astimezone().tzinfo


def get_user_stat_type(value):
    if value == 'int':
        return VALUETYPE_INT
    elif value == 'float':
        return VALUETYPE_FLOAT
    else:
        return None


class TokenManager:
    def __init__(self, token, refresh_token, user_id):
        self.access_token = token
        self.refresh_token = refresh_token
        self.user_id = user_id
        self.session = None

        self.tokens = dict()
        self.achievements = list()
        self.client_id = None
        self.client_secret = None

    async def setup(self):
        self.session = aiohttp.ClientSession(headers={'Authorization': f'Bearer {self.access_token}'})

    async def obtain_token_for(self, client_id, client_secret):
        self.client_id = client_id
        self.client_secret = client_secret
        response = await self.session.get(
            f"https://auth.gog.com/token?"
            f"client_id={client_id}&"
            f"client_secret={client_secret}&"
            f"grant_type=refresh_token&"
            f"refresh_token={self.refresh_token}&"
            f"without_new_session=1"
        )

        if not response.ok:
            print("Error obtaining access token")
            return None
        json_data = await response.json()
        json_data["comet_obtain_time"] = time.time()
        self.tokens[client_id] = json_data
        return json_data

    async def refresh_token_for(self, client_id, client_secret):
        token = self.tokens[client_id]
        if token['comet_obtain_time'] + token["expires_in"] >= time.time():
            return False, None

        response = await self.session.get(
            f"https://auth.gog.com/token?"
            f"client_id={client_id}&"
            f"client_secret={client_secret}&"
            f"grant_type=refresh_token&"
            f"refresh_token={token['refresh_token']}&"
            f"without_new_session=1"
        )

        if not response.ok:
            print("Error refreshing access token")
            return False, None
        json_data = await response.json()
        json_data["comet_obtain_time"] = time.time()
        self.tokens[client_id] = json_data
        return True, json_data

    async def get_user_info(self):
        response = await self.session.get("https://embed.gog.com/userData.json")

        if not response.ok:
            print("Error obtaining user data")
            return None

        data = await response.json()
        return data

    async def get_info_for_users(self, ids):
        string_ids = ",".join(ids)

        url = f"https://users.gog.com/users?ids={string_ids}"

        token = self.tokens.get(self.client_id)

        res = await self.session.get(url, headers={"Authorization": f"Bearer {token['access_token']}"})

        data = await res.json()
        return data["items"]

    async def get_user_stats(self, user_id):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user stats requested before token")
        response = await self.session.get(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{user_id}/stats",
            headers={"Authorization": f"Bearer {token['access_token']}"}
        )

        if not response.ok:
            print("Error obtaining user stats")
            return None

        data = await response.json()

        array = data['items']
        stats_list = list()
        for statObj in array:
            stat = GogUserStat()

            stat.stat_id = statObj['stat_id']
            stat.stat_key = statObj['stat_key']
            stat.stat_type = get_user_stat_type(statObj['type'])

            if statObj.get('window'):
                stat.window_size = statObj['window']

            stat.increment_only = statObj['increment_only']

            if stat.stat_type == VALUETYPE_INT:
                stat.value.i = statObj['value']

                if statObj.get("default_value"):
                    stat.default_value.i = statObj["default_value"]
                else:
                    stat.default_value.i = 0

                if statObj.get("min_value"):
                    stat.min_value.i = statObj["min_value"]
                else:
                    stat.min_value.i = 0

                if statObj.get("max_value"):
                    stat.max_value.i = statObj["max_value"]
                else:
                    stat.max_value.i = 1000000

                if statObj["max_change"]:
                    stat.max_change.i = statObj["max_change"]
                else:
                    stat.max_change.i = 1

            stats_list.append(stat)
        return stats_list

    async def update_user_stat(self, stat_id, value):
        token = self.tokens.get(self.client_id)
        response = await self.session.post(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{self.user_id}/stats/{stat_id}",
            json={
                'value': value
            },

            headers={'Authorization': f"Bearer {token['access_token']}"})
        return response.ok, response.status

    async def delete_user_stats(self):
        token = self.tokens.get(self.client_id)

        response = await self.session.delete(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{self.user_id}/stats",
            headers={'Authorization': f"Bearer {token['access_token']}"})

        if not response.ok:
            print("Error deleting achievements")
            return response.status
        return 202

    async def get_user_achievements(self, user_id):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user achievements requested before token")

        response = await self.session.get(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{user_id}/achievements",
            headers={'Authorization': f'Bearer {token["access_token"]}'}
        )

        json_data = await response.json()
        achievement_array = json_data['items']
        self.achievements = achievement_array
        achievements = UserAchievementList()
        for achievement_obj in achievement_array:
            achievement = UserAchievement()

            achievement.achievement_id = int(achievement_obj['achievement_id'])
            achievement.achievement_key = achievement_obj['achievement_key']
            achievement.name = achievement_obj['name']
            achievement.description = achievement_obj['description']
            achievement.image_url_locked = achievement_obj['image_url_locked']
            achievement.image_url_unlocked = achievement_obj['image_url_unlocked']
            achievement.visible_while_locked = achievement_obj['visible']

            if type(achievement_obj["date_unlocked"]) == str:
                date_str = achievement_obj["date_unlocked"]
                time = datetime.datetime.fromisoformat(date_str.replace("+0000", "+00:00"))
                achievement.unlock_time = int(time.timestamp())
            else:
                achievement.unlock_time = 0

            achievement.rarity = achievement_obj["rarity"]
            achievement.rarity_desc = achievement_obj["rarity_level_description"]
            achievement.rarity_slug = achievement_obj["rarity_level_slug"]

            achievements.items.append(achievement)
        achievements.mode = json_data['achievements_mode']
        achievements.language = 'en-US'
        return achievements

    async def set_user_achievement(self, ach_id, time):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user achievement unlock requested before token")
            return

        is_unlocked_already = False

        # Prevent setting achievements again
        if time != 0:
            for achievement in self.achievements:
                if int(achievement["achievement_id"]) == ach_id:
                    is_unlocked_already = type(achievement["date_unlocked"]) == str

        if is_unlocked_already:
            return True, True

        date_unlocked = None

        if time != 0:
            date_unlocked = datetime.datetime.fromtimestamp(time, tz=LOCAL_TIMEZONE).astimezone(
                datetime.timezone.utc).strftime(
                "%Y-%m-%dT%H:%M:%S+0000")

        payload = {
            "date_unlocked": date_unlocked
        }

        response = await self.session.post(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{self.user_id}/achievements/{ach_id}",
            json=payload,
            headers={'Authorization': f'Bearer {token["access_token"]}'}
        )

        if not response.ok:
            print("Error unlocking user achievement")
            print(response.content)
            return False, False

        return False, True  # is_unlocked, is_ok

    async def delete_user_achievements(self):
        token = self.tokens.get(self.client_id)

        response = await self.session.delete(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{self.user_id}/achievements",
            headers={'Authorization': f"Bearer {token['access_token']}"})

        if not response.ok:
            print("Error deleting achievements")
            return response.status
        return 202

    async def get_leaderboards(self):
        request_url = f"https://gameplay.gog.com/clients/{self.client_id}/leaderboards"

        token = self.tokens.get(self.client_id)
        response = await self.session.get(request_url, headers={'Authorization': f'Bearer {token["access_token"]}'})

        data = await response.json()

        leaderboards_definitions = list()
        for leaderboard in data["items"]:
            definition = LeaderboardDefinition()
            definition.leaderboard_id = int(leaderboard["id"])
            definition.name = leaderboard["name"]
            definition.key = leaderboard["key"]

            l_sort_method = leaderboard["sort_method"]
            definition.sort_method = communication_service_pb2.SortMethod.SORT_METHOD_UNDEFINED
            if l_sort_method == "desc":
                definition.sort_method = communication_service_pb2.SortMethod.SORT_METHOD_DESCENDING
            elif l_sort_method == "asc":
                definition.sort_method = communication_service_pb2.SortMethod.SORT_METHOD_ASCENDING

            l_display_type = leaderboard["display_type"]
            definition.display_type = communication_service_pb2.DisplayType.DISPLAY_TYPE_UNDEFINED

            if l_display_type == "numeric":
                definition.display_type = communication_service_pb2.DisplayType.DISPLAY_TYPE_NUMERIC
            elif l_display_type == "time_seconds":
                definition.display_type = communication_service_pb2.DisplayType.DISPLAY_TYPE_TIME_SECONDS
            elif l_display_type == "time_milliseconds":
                definition.display_type = communication_service_pb2.DisplayType.DISPLAY_TYPE_TIME_MILLISECONDS

            leaderboards_definitions.append(definition)

        return leaderboards_definitions

    async def get_leaderboard_entries(self, leaderboard_id, range_start=None, range_end=None, user_id=None,
                                      count_before=None,
                                      count_after=None):

        params = dict()

        if range_start is not None:
            params['range_start'] = range_start
        if range_end is not None:
            params['range_end'] = range_end
        if user_id is not None:
            params['user'] = user_id
        if count_after is not None:
            params['count_after'] = count_after
        if count_before is not None:
            params['count_before'] = count_before

        url = f"https://gameplay.gog.com/clients/{self.client_id}/leaderboards/{leaderboard_id}/entries?"
        url = url + parse.urlencode(params)

        print(url)

        token = self.tokens.get(self.client_id)
        response = await self.session.get(url, headers={'Authorization': f"Bearer {token['access_token']}"})

        if not response.ok:
            return [], 0, response.status

        data = await response.json()

        leaderboard_entries = list()
        for entry in data['items']:
            leaderboard_entry = LeaderboardEntry()
            # Protobuf encoding: I64 fixed64, sfixed64, double
            # FIXME: figure out this encoding, and why it's needed
            leaderboard_entry.user_id = 0x200000000000000 | int(entry["user_id"])
            leaderboard_entry.rank = int(entry["rank"])
            leaderboard_entry.score = int(entry["score"])

            leaderboard_entries.append(leaderboard_entry)
        return leaderboard_entries, data['leaderboard_entry_total_count'], response.status
