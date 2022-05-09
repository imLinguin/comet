class GogUserStat():
    def __init__(self):
        self.stat_id = 0
        self.stat_key = ''
        self.stat_type = VALUETYPE_UNDEFINED
        self.window_size = 0 
        self.increment_only = False

        self.value = FloatInt()
        self.default_value =FloatInt() 
        self.min_value =FloatInt()
        self.max_value=FloatInt()
        self.max_change=FloatInt()


class FloatInt():
    def __init__(self):
        i=0
        f=0.0

        
VALUETYPE_UNDEFINED = 0
VALUETYPE_INT = 1
VALUETYPE_FLOAT =2 
VALUETYPE_NONE =3

