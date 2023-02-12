from dataclasses import dataclass


@dataclass
class LeaderboardDefinition:
    leaderboard_id = 0
    key = ''
    name = ''
    sort_method = 0
    display_type = 0


@dataclass
class LeaderboardEntry:
    rank = 0
    score = 0
    user_id = ""
    details = ""
