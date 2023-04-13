import pandas as pd

def bar(df):
    df["sum_field1"] = df["field1"].sum()
    return df
