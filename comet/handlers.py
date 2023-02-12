import logging
import socket
from dataclasses import dataclass

from comet.api.token import TokenManager
from comet.proto.gog.protocols import pb_pb2
from comet.proto.galaxy.protocols import communication_service_pb2, webbroker_service_pb2

import time

SORT_COMM = 1
SORT_WEBBROKER = 2

UNKNOWN_MESSAGE = 0
LIBRARY_INFO_REQUEST = 1
LIBRARY_INFO_RESPONSE = 2
AUTH_INFO_REQUEST = 3
AUTH_INFO_RESPONSE = 4
GET_USER_STATS_REQUEST = 15
GET_USER_STATS_RESPONSE = 16
UPDATE_USER_STAT_REQUEST = 17
UPDATE_USER_STAT_RESPONSE = 18
DELETE_USER_STATS_REQUEST = 19
DELETE_USER_STATS_RESPONSE = 20
GET_GLOBAL_STATS_REQUEST = 21
GET_GLOBAL_STATS_RESPONSE = 22
GET_USER_ACHIEVEMENTS_REQUEST = 23
GET_USER_ACHIEVEMENTS_RESPONSE = 24
UNLOCK_USER_ACHIEVEMENT_REQUEST = 25
UNLOCK_USER_ACHIEVEMENT_RESPONSE = 26
CLEAR_USER_ACHIEVEMENT_REQUEST = 27
CLEAR_USER_ACHIEVEMENT_RESPONSE = 28
DELETE_USER_ACHIEVEMENTS_REQUEST = 29
DELETE_USER_ACHIEVEMENTS_RESPONSE = 30
GET_LEADERBOARDS_REQUEST = 31
GET_LEADERBOARDS_RESPONSE = 32
GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST = 33
GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST = 34
GET_LEADERBOARD_ENTRIES_FOR_USERS_REQUEST = 35
GET_LEADERBOARD_ENTRIES_RESPONSE = 36
SET_LEADERBOARD_SCORE_REQUEST = 37
SET_LEADERBOARD_SCORE_RESPONSE = 38
AUTH_STATE_CHANGE_NOTIFICATION = 39
GET_LEADERBOARDS_BY_KEY_REQUEST = 40
CREATE_LEADERBOARD_REQUEST = 41
CREATE_LEADERBOARD_RESPONSE = 42
GET_USER_TIME_PLAYED_REQUEST = 43
GET_USER_TIME_PLAYED_RESPONSE = 44
SHARE_FILE_REQUEST = 47
SHARE_FILE_RESPONSE = 48
START_OVERLAY_SESSION_REQUEST = 49
START_OVERLAY_SESSION_RESPONSE = 50
START_GAME_SESSION_REQUEST = 49
START_GAME_SESSION_RESPONSE = 50
CONFIGURE_ENVIRONMENT_REQUEST = 59
CONFIGURE_ENVIRONMENT_RESPONSE = 60

AUTH_REQUEST = 1
AUTH_RESPONSE = 2
SUBSCRIBE_TOPIC_REQUEST = 3
SUBSCRIBE_TOPIC_RESPONSE = 4
MESSAGE_FROM_TOPIC = 5


@dataclass
class HandlerResponse:
    header = pb_pb2.Header()
    data = bytes()


def message_id(sort, msg_type):
    return (sort << 16) | msg_type


@dataclass
class ConnectionHandler:
    connection: socket.socket
    address: str
    token_manager: TokenManager
    data = bytes()
    closed = False

    logger = logging.getLogger("handler")

    def handle_conection(self):
        while not self.closed:
            try:
                header_size_bytes = self.connection.recv(2)
                if not header_size_bytes:
                    time.sleep(0.1)
                    continue
            except OSError:
                self.logger.error("handle_connection:Error reading socket data")
                self.closed = True
                return

            try:
                self.handle_message(header_size_bytes)
            except:
                self.connection.close()
                raise

    def handle_message(self, size):
        header_size = int.from_bytes(size, 'big')

        header_data = self.connection.recv(header_size)

        header = pb_pb2.Header()
        header.ParseFromString(header_data)

        message_data = self.connection.recv(header.size)
        self.logger.info(f"handle_message:Header {header.sort}|{header.type}")

        combined_id = message_id(header.sort, header.type)
        # ———————————No switches?———————————
        # ⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
        # ⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
        # ⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
        # ⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
        # ⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
        # ⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
        # ⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        # ⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
        # —————————————————————————————
        if combined_id == message_id(SORT_COMM, AUTH_INFO_REQUEST):
            res = self.handle_auth_request(message_data)
        elif combined_id == message_id(SORT_COMM, GET_USER_STATS_REQUEST):
            res = self.handle_user_stats_request(message_data)
        elif combined_id == message_id(SORT_COMM, GET_USER_ACHIEVEMENTS_REQUEST):
            res = self.handle_get_user_achievements(message_data)
        elif combined_id == message_id(SORT_COMM, UNLOCK_USER_ACHIEVEMENT_REQUEST):
            res = self.handle_unlock_user_achievement(message_data)
        elif combined_id == message_id(SORT_COMM, CLEAR_USER_ACHIEVEMENT_REQUEST):
            res = self.handle_clear_user_achievement(message_data)
        elif combined_id == message_id(SORT_COMM, GET_LEADERBOARDS_REQUEST):
            res = self.handle_get_leaderboards(message_data)
        elif combined_id == message_id(SORT_COMM, GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST):
            res = self.handle_get_leaderboard_entries_global(message_data)
        elif combined_id == message_id(SORT_COMM, GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST):
            res = self.handle_get_leaderboard_entries_arround_user(message_data)
        elif combined_id == message_id(SORT_WEBBROKER, SUBSCRIBE_TOPIC_REQUEST):
            res = self.handle_subscribe_topic(message_data)
        else:
            self.logger.warning(f"handle_message:fixme:unknown call {header.sort}|{header.type}")
            print(message_data)
            return

        if res:
            res.header.size = len(res.data)
            if header.HasField("oseq"):
                res.header.Extensions[pb_pb2.Response.rseq] = header.oseq

            res_header_data = res.header.SerializeToString()
            res_header_data_size = len(res_header_data).to_bytes(2, 'big')

            self.connection.sendmsg([res_header_data_size, res_header_data, res.data])

            self.logger.info(f"handle_message:responded with {res.header.sort}|{res.header.type}")

    def handle_auth_request(self, data):
        msg = communication_service_pb2.AuthInfoRequest()
        msg.ParseFromString(data)

        credentials = self.token_manager.obtain_token_for(msg.client_id, msg.client_secret)
        user_info = self.token_manager.get_user_info()

        res_data = communication_service_pb2.AuthInfoResponse()

        res_data.refresh_token = credentials["refresh_token"]
        res_data.environment_type = 0
        res_data.user_id = int(user_info["userId"])
        res_data.user_name = user_info["username"]
        res_data.region = 0

        self.logger.info(f"handle_auth_request:authenticated the user {user_info['username']}")
        res = HandlerResponse()

        res.data = res_data.SerializeToString()

        res.header.sort = SORT_COMM
        res.header.type = AUTH_INFO_RESPONSE
        return res

    def handle_subscribe_topic(self, data):
        msg = webbroker_service_pb2.SubscribeTopicRequest()
        msg.ParseFromString(data)

        response = webbroker_service_pb2.SubscribeTopicResponse()
        response.topic = msg.topic

        self.logger.info(f"handle_subscribe_topic:stub:{msg.topic}")

        res = HandlerResponse()
        res.data = response.SerializeToString()

        res.header.sort = SORT_WEBBROKER
        res.header.type = SUBSCRIBE_TOPIC_RESPONSE

        return res

    def handle_user_stats_request(self, data):
        msg = communication_service_pb2.GetUserStatsRequest()
        msg.ParseFromString(data)

        stats = self.token_manager.get_user_stats(msg.user_id)

        if not stats:
            return None

        response = communication_service_pb2.GetUserStatsResponse()

        for stat in stats:
            stat_pb = communication_service_pb2.GetUserStatsResponse.UserStat()

            stat_pb.stat_id = int(stat.stat_id)
            stat_pb.key = stat.stat_key
            stat_pb.value_type = stat.stat_type
            stat_pb.window_size = stat.window_size
            stat_pb.increment_only = stat.increment_only

            if stat.stat_type == 1:
                stat_pb.int_value = stat.value.i
                stat_pb.int_default_value = stat.default_value.i
                stat_pb.int_min_value = stat.min_value.i
                stat_pb.int_max_value = stat.max_value.i
                stat_pb.int_max_change = stat.max_change.i
            elif stat.stat_type == 2:
                stat_pb.float_value = stat.value.f
                stat_pb.float_default_value = stat.default_value.f
                stat_pb.float_min_value = stat.min_value.f
                stat_pb.float_max_value = stat.max_value.f
                stat_pb.float_max_change = stat.max_change.f

            response.user_stats.append(stat_pb)

        res = HandlerResponse()

        res.data = response.SerializeToString()

        res.header.sort = SORT_COMM
        res.header.type = GET_USER_STATS_RESPONSE
        return res

    def handle_get_user_achievements(self, data):
        msg = communication_service_pb2.GetUserAchievementsRequest()
        msg.ParseFromString(data)

        achievements = self.token_manager.get_user_achievements(msg.user_id)

        response = communication_service_pb2.GetUserAchievementsResponse()
        for achievement in achievements.items:
            achievement_pb = communication_service_pb2.GetUserAchievementsResponse.UserAchievement()

            achievement_pb.achievement_id = achievement.achievement_id
            achievement_pb.key = achievement.achievement_key
            achievement_pb.name = achievement.name
            achievement_pb.description = achievement.description
            achievement_pb.image_url_locked = achievement.image_url_locked
            achievement_pb.image_url_unlocked = achievement.image_url_unlocked
            achievement_pb.visible_while_locked = achievement.visible_while_locked
            if achievement.unlock_time != 0:
                achievement_pb.unlock_time = achievement.unlock_time
            achievement_pb.rarity = achievement.rarity
            achievement_pb.rarity_level_description = achievement.rarity_desc
            achievement_pb.rarity_level_slug = achievement.rarity_slug

        response.language = achievements.language
        response.achievements_mode = achievements.mode

        res = HandlerResponse()

        res.data = response.SerializeToString()

        res.header.sort = SORT_COMM
        res.header.type = GET_USER_ACHIEVEMENTS_RESPONSE
        return res

    def handle_unlock_user_achievement(self, data):
        msg = communication_service_pb2.UnlockUserAchievementRequest()
        msg.ParseFromString(data)

        self.token_manager.set_user_achievement(msg.achievement_id, msg.time)

        res = HandlerResponse()

        res.data = bytes()

        res.header.sort = SORT_COMM
        res.header.type = UNLOCK_USER_ACHIEVEMENT_RESPONSE
        return res

    def handle_clear_user_achievement(self, data):
        msg = communication_service_pb2.ClearUserAchievementRequest()
        msg.ParseFromString(data)

        self.token_manager.set_user_achievement(msg.achievement_id, 0)
        res = HandlerResponse()

        res.data = bytes()

        res.header.sort = SORT_COMM
        res.header.type = CLEAR_USER_ACHIEVEMENT_RESPONSE
        return res

    def handle_get_leaderboards(self, data):
        leaderboards = self.token_manager.get_leaderboards()

        leaderboards_data = communication_service_pb2.GetLeaderboardsResponse()
        for leaderboard in leaderboards:
            pb_leaderboard = communication_service_pb2.GetLeaderboardsResponse.LeaderboardDefinition()

            pb_leaderboard.leaderboard_id = leaderboard.leaderboard_id
            pb_leaderboard.key = leaderboard.key
            pb_leaderboard.name = leaderboard.name
            pb_leaderboard.sort_method = leaderboard.sort_method
            pb_leaderboard.display_type = leaderboard.display_type

            leaderboards_data.leaderboard_definitions.append(pb_leaderboard)

        res = HandlerResponse()
        res.data = leaderboards_data.SerializeToString()

        res.header.sort = SORT_COMM
        res.header.type = GET_LEADERBOARDS_RESPONSE

        return res

    def __prepare_leaderboard_entries_response(self, entries):
        leaderboards_entries_res = communication_service_pb2.GetLeaderboardEntriesResponse()
        for entry in entries:
            pb_entry = communication_service_pb2.GetLeaderboardEntriesResponse.LeaderboardEntry()
            pb_entry.rank = entry.rank
            pb_entry.score = entry.score
            pb_entry.user_id = entry.user_id
            leaderboards_entries_res.leaderboard_entries.append(pb_entry)

        return leaderboards_entries_res

    def handle_get_leaderboard_entries_global(self, data):
        msg = communication_service_pb2.GetLeaderboardEntriesGlobalRequest()
        msg.ParseFromString(data)

        entries, total, status = self.token_manager.get_leaderboard_entries(msg.leaderboard_id,
                                                                            range_start=msg.range_start,
                                                                            range_end=msg.range_end)

        res = HandlerResponse()
        res.data = bytes()

        res.header.sort = SORT_COMM
        res.header.type = GET_LEADERBOARD_ENTRIES_RESPONSE

        if not entries:
            self.logger.info(f"leaderboards:status code:{status}")
            res.header.Extensions[pb_pb2.Response.code] = status
            return res

        leaderboards_entries_res = self.__prepare_leaderboard_entries_response(entries)

        leaderboards_entries_res.leaderboard_entry_total_count = total

        res.data = leaderboards_entries_res.SerializeToString()

        return res

    def handle_get_leaderboard_entries_arround_user(self, data):
        msg = communication_service_pb2.GetLeaderboardEntriesAroundUserRequest()
        msg.ParseFromString(data)

        user_id = int(bin(msg.user_id)[4:], 2)  # Stip first two bits see token_manager.get_leaderboard_entries

        entries, total, status = self.token_manager.get_leaderboard_entries(msg.leaderboard_id, user_id=user_id,
                                                                            count_before=msg.count_before,
                                                                            count_after=msg.count_after)

        res = HandlerResponse()
        res.data = bytes()

        res.header.sort = SORT_COMM
        res.header.type = GET_LEADERBOARD_ENTRIES_RESPONSE

        if not entries:
            self.logger.info(f"leaderboards:status code:{status}")
            res.header.Extensions[pb_pb2.Response.code] = status
            return res

        leaderboards_entries_res = self.__prepare_leaderboard_entries_response(entries)

        leaderboards_entries_res.leaderboard_entry_total_count = total

        res.data = leaderboards_entries_res.SerializeToString()

        return res
