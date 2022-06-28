from dataclasses import dataclass

VALUETYPE_UNDEFINED = 0
VALUETYPE_INT = 1
VALUETYPE_FLOAT = 2
VALUETYPE_NONE = 3


@dataclass
class FloatInt:
    i = 0
    f = 0.0


@dataclass
class GogUserStat:
    stat_id = 0
    stat_key = ''
    stat_type = VALUETYPE_UNDEFINED
    window_size = 0
    increment_only = False

    value = FloatInt()
    default_value = FloatInt()
    min_value = FloatInt()
    max_value = FloatInt()
    max_change = FloatInt()
