class UserAchievement():
    def __init__(self):
        self.achievement_id = 0
        self.achievement_key = ''
        self.name = ''
        self.description = ''
        self.image_url_locked = ''
        self.image_url_unlocked = ''
        self.visible_while_locked = False
        self.unlock_time = 0
        self.rarity = 0.0
        self.rarity_desc = ''
        self.rarity_slug = ''

class UserAchievementList():
    def __init__(self):
        self.items = list()
        self.mode = ''
        self.language = ''
