import pandas as pd

COLUMNS = [
    "time",
    "thrust",
    "cmd_roll",
    "cmd_pitch",
    "cmd_yaw",
    "roll",
    "pitch",
    "yaw",
    "vroll",
    "vpitch",
    "vyaw",
    "hthrust",
    "pid_roll",
    "pid_pitch",
    "pid_yaw",
    "pid_vroll",
    "pid_vpitch",
    "pid_vyaw",
]


def to_dataframe(iterable):
    df = pd.DataFrame(columns=COLUMNS)
    for entry in iterable:
        if entry.size == 0:
            break
        tmp = pd.DataFrame(entry.T, columns=COLUMNS)
        df = pd.concat([df, tmp])
    df.set_index("time", inplace=True)
    return df
