#!/usr/bin/env python3
import numpy as np

"""
Drone constants
"""

G           = 9.8       # g-force [m/s²]
RHO_0       = 1.293     # standard air density [kg/m³]
P_0         = 101325    # standard atmospheric pressure, Pa
T           = 298       # °K (25°C)
H           = 200       # m
P           = P_0 * (1 - 0.0065 * H / T)**5.2561    # atmospheric pressure, Pa
RHO         = 273 * P / (P_0 * T) * RHO_0           # air density, kg/m³
INCH_2_M    = 0.0254
RADPS_2_RPM = 60 / (2 * np.pi)
RPM_2_RADPS = 1 / RADPS_2_RPM
