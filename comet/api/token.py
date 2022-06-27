import datetime

import requests

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
    def __init__(self, args):
        self.access_token = args.token
        self.refresh_token = args.refresh_token
        self.user_id = args.user_id
        self.session = requests.session()
        self.session.headers = {
            "Authorization": f"Bearer {self.access_token}"
        }

        self.tokens = dict()
        self.client_id = None
        self.client_secret = None

    def obtain_token_for(self, client_id, client_secret):
        self.client_id = client_id
        self.client_secret = client_secret
        response = self.session.get(
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
        json_data = response.json()
        self.tokens[client_id] = json_data
        return json_data

    def get_user_info(self):
        response = self.session.get("https://embed.gog.com/userData.json")

        if not response.ok:
            print("Error obtaining user data")
            return None

        return response.json()

    def get_user_stats(self, user_id):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user stats requested before token")
        response = self.session.get(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{user_id}/stats",
            headers={"Authorization": f"Bearer {token['access_token']}"}
        )

        if not response.ok:
            print("Error obtaining user stats")
            return None

        data = response.json()

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
    
    def get_user_achievements(self, user_id):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user achievements requested before token")

        response = self.session.get(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{user_id}/achievements",
            headers={'Authorization': f'Bearer {token["access_token"]}'}
        )

        json_data = response.json()
        achievement_array = json_data['items']
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
                time = datetime.datetime.fromisoformat(date_str)
                achievement.unlock_time = time.timestamp()*1000
            else:
                achievement.unlock_time = 0
            
            achievement.rarity = achievement_obj["rarity"]
            achievement.rarity_desc = achievement_obj["rarity_level_description"]
            achievement.rarity_slug = achievement_obj["rarity_level_slug"]

            achievements.items.append(achievement)
        achievements.mode = json_data['achievements_mode']
        achievements.language = 'en-US'
        return achievements

    def set_user_achievement(self, ach_id, time):
        token = self.tokens.get(self.client_id)
        if not token:
            print("Error, user achievement unlock requested before token")
            return
        payload = {
            "date_unlocked": datetime.datetime.fromtimestamp(
                time/1000, tz=LOCAL_TIMEZONE
            ).astimezone(datetime.timezone.utc).isoformat(timespec="seconds")
        }

        response = self.session.post(
            f"https://gameplay.gog.com/clients/{self.client_id}/users/{self.user_id}/achievements/{ach_id}",
            data=payload,
            headers={'Authorization': f'Bearer {token["access_token"]}'}
        )

        if not response.ok:
            print("Error unlocking user achievement")
            return False
        
        return True
