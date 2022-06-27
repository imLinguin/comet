from dataclasses import dataclass, field


@dataclass
class UserAchievement:
    achievement_id = 0
    achievement_key = ''
    name = ''
    description = ''
    image_url_locked = ''
    image_url_unlocked = ''
    visible_while_locked = False
    unlock_time = 0
    rarity = 0.0
    rarity_desc = ''
    rarity_slug = ''


@dataclass
class UserAchievementList:
    items: list = field(default_factory=list)
    mode = ''
    language = ''
