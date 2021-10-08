#!/usr/bin/env python3
import toml
import numpy as np
import sympy as sp
import control
import constants

"""
Quadcopter parameters
"""
# Frame
F_MASS      = 0.9       # kg
F_SIZE      = 450       # mm

# Motor
M_MASS      = 52        # g
M_KV        = 750       # rmp/V
M_P_MAX     = 185       # W
M_R         = 0.192     # Ohm
M_SIZE_1    = 28        # mm
M_SIZE_2    = 30        # mm
M_U_NOM     = 10        # V
M_I_NOM     = 0.5       # A
M_I_MAX     = 19        # A

# ESC
ESC_R       = 0.008     # Ohm

# Propeller
P_DIAMETER  = 0.254     # m (10")
P_PITCH     = 0.1016    # m (4.5")
P_BLADE     = 2

# Battery
B_CELL      = 3
B_CAPACITY  = 2600      # mAh
B_C         = 45        # C
B_R         = 0.0063    # Ohm

"""
Generic parameter
"""
G           = 9.8       # m/s²
RHO_0       = 1.293     # standard air density, kg/m³
P_0         = 101325    # standard atmospheric pressure, Pa
T           = 298       # °K (25°C)
H           = 200       # m
P           = P_0 * (1 - 0.0065 * H / T)**5.2561    # atmospheric pressure, Pa
RHO         = 273 * P / (P_0 * T) * RHO_0           # air density, kg/m³

S = control.tf([1, 0], 1)


class Propeller:
    """
    Propeller
    """
    def __init__(self, diameter, pitch, blade, mass=None):
        """
        Create a new propeller with
        - ```diameter``` in inch
        - ```pitch``` in inch
        - ```blade``` number
        - ```mass``` in kg (optional)
        """
        self.diameter = diameter * constants.INCH_2_M
        self.pitch = pitch * constants.INCH_2_M
        self.blade = blade
        self._compute_coeff()

    def _compute_coeff(self):
        A = 5
        EPSILON = 0.85
        LAMBDA = 0.75
        ZETA = 0.5
        e = 0.83
        Cfd = 0.015
        K0 = 6.11

        C1 = np.pi * A * K0 * EPSILON * np.arctan(self.pitch / (self.diameter * np.pi)) / (np.pi * A + K0)
        Cd = Cfd + 1 / (np.pi * A * e) * C1**2
        Ct = 0.25 * np.pi**3 * LAMBDA * ZETA**2 * self.blade * C1 / (np.pi * A)
        Cm = 1 / (8 * A) * np.pi**2 * Cd * ZETA**2 * LAMBDA * self.blade**2
        self.Ct = Ct * RHO * self.diameter**4 / 60**2
        self.Cm = Cm * RHO * self.diameter**5 / 60**2

    def thrust(self, rpm):
        """
        Compute the propeller thrust in N for the given ```rpm```
        """
        return self.Ct * rpm**2

    def torque(self, rpm):
        """
        Compute the propeller torque in N.m for the given ```rpm```
        """
        return self.Cm * rpm**2

    def rpm(self, thrust):
        return np.sqrt(thrust / self.Ct)


class Motor:
    """
    Brushless DC motor
    """
    def __init__(self, kv, i_max, i_nom, u_nom, rm, mass=None):
        """
        Create a new motor with
        - ```kv```
        - ```i_max``` in A
        - ```i_nom``` in A, the nominal current with no load
        - ```u_nom``` in V, the nominal voltage with no load
        - ```rm``` in Ohm
        - ```mass``` in kg (optional)
        """
        self.kv = kv
        self.i_max = i_max
        self.i_nom = i_nom
        self.u_nom = u_nom
        self.rm = rm
        self.mass = mass

    def current(self, torque):
        return torque * self.kv * self.u_nom / (9.55 * (self.u_nom - self.i_nom * self.rm)) + self.i_nom

    def torque(self, i_m):
        return (i_m - self.i_nom) * (9.55 * (self.u_nom - self.i_nom * self.rm)) / (self.kv * self.u_nom)

    def voltage(self, rpm, torque):
        return self.current(torque) * self.rm + (self.u_nom - self.i_nom * self.rm) / (self.kv * self.u_nom) * rpm

    def rpm_s(self, i_mot, u_mot):
        return (u_mot - i_mot * self.rm) * (self.kv * self.u_nom) / (self.u_nom - self.i_nom * self.rm)

    def rpm(self, throttle):
        Cr = 1
        Tm = 2
        wb = 3
        return (Cr * throttle + wb) / (Tm * S + 1)


class ESC:
    """
    Electronic Speed Controller
    """
    def __init__(self, r, mass=None):
        """
        Create a new ESC with
        - ```r``` in Ohm
        - ```mass``` in kg (optional)
        """
        self.r = r
        self.mass = mass

    def throttle(self, u_mot, i_mot, u_bat):
        return (u_mot + self.r * i_mot) / u_bat

    def current(self, u_mot, i_mot, u_bat):
        return self.throttle(u_mot, i_mot, u_bat) * i_mot


class Battery:
    """
    Battery
    """
    def __init__(self, capacity, cell, r, discharge_rate, mass=None):
        """
        Create a new battery with
        - ```capacity``` in mAh
        - ```cell```
        - ̀```r``` in Ohm
        - ```discharge_rate```
        - ```mass``` (optional)
        """
        self.capacity = capacity
        self.u_bat = cell * 4.2
        self.r = r
        self.discharge_rate = discharge_rate
        self.mass = mass

    def endurance(self, i_bat):
        return 0.8 * self.capacity / i_bat * 60 / 1000

    def voltage(self, i_bat):
        return self.u_bat - i_bat * self.r


class Drone:
    """
    Full drone
    """
    def __init__(self, propeller, motor, esc, battery, arms, mass=None):
        """
        Create a new drone
        """
        self.propeller = propeller
        self.motor = motor
        self.esc = esc
        self.battery = battery
        self.arms = arms
        self.mass = mass

        self.w_hover = None
        self.throttle_hover = None
        self.w_max = None

    def hovering(self):
        """
        Gives drone performance in hovering mode
        """
        n_mot = self.propeller.rpm(self.mass * G / self.arms)
        self.w_hover = n_mot * constants.RPM_2_RADPS
        m_mot = self.propeller.torque(n_mot)
        u_mot = self.motor.voltage(n_mot, m_mot)
        i_mot = self.motor.current(m_mot)
        self.throttle_hover = self.esc.throttle(u_mot, i_mot, self.battery.u_bat)
        i_esc = self.throttle_hover * i_mot
        i_bat = self.arms * i_esc + 1
        u_esc = self.battery.voltage(i_bat)
        t = self.battery.endurance(i_bat)
        print('In hover mode:')
        print(f'\t- the drone can fly up to {t:.2f} minutes')
        print(f'\t- the throttle command is {self.throttle_hover*100:.2f}%')
        print(f'\t- the ESCs input current is {i_esc:.2f}A')
        print(f'\t- the ESCs input voltage is {u_esc:.2f}V')
        print(f'\t- the battery current is {i_bat:.2f}A')
        print(f'\t- the motor speed is {n_mot:.0f} rpm')

    def full_throtlle(self):
        """
        Gives drone performance in full throttle mode
        """
        n_mot = sp.symbols('N')
        expr = self.motor.current(self.propeller.torque(n_mot)) * self.esc.r + \
            self.motor.voltage(n_mot, self.propeller.torque(n_mot)) - \
            self.battery.u_bat
        n_mot = max(sp.roots(expr, n_mot))
        self.w_max = n_mot * constants.RPM_2_RADPS
        m_mot = self.propeller.torque(n_mot)
        i_mot = self.motor.current(m_mot)
        i_bat = self.arms * i_mot + 1
        u_esc = self.battery.voltage(i_bat)
        efficiency = ((2 * np.pi / 60) * self.arms * m_mot * n_mot) / (i_bat * self.battery.u_bat)
        print('In full throttle mode:')
        print(f'\t- the ESCs input current is {i_mot:.2f}V')
        print(f'\t- the ESCs input voltage is {u_esc:.2f}V')
        print(f'\t- the battery current is {i_bat:.2f}A')
        print(f'\t- the motor speed is {n_mot:.0f} rpm')
        print(f'\t- the drone efficiency is {efficiency * 100:.2f}%')

    def forward_flight(self):
        n_mot = sp.symbols('N')
        expr = self.motor.current(self.propeller.torque(n_mot)) * self.esc.r + \
            self.motor.voltage(n_mot, self.propeller.torque(n_mot)) - \
            0.8 * self.battery.u_bat
        n_mot = float(max(sp.roots(expr, n_mot)))
        thrust = self.propeller.thrust(n_mot)
        max_load = thrust * self.arms - self.mass * G
        max_pitch = np.arccos(self.mass * G / (self.arms * thrust))

        s = (self.propeller.diameter / 2)**2 * np.pi
        cd = 3 * (1 - np.sin(max_pitch)**3) + 1.5 * (1 - np.cos(max_pitch)**3)
        v_max = np.sqrt((2 * self.mass * G * np.tan(max_pitch)) / (RHO * s * cd))
        n_mot = self.propeller.rpm(self.mass * G / (np.cos(max_pitch) * self.arms))
        m_mot = self.propeller.torque(n_mot)
        u_mot = self.motor.voltage(n_mot, m_mot)
        i_mot = self.motor.current(m_mot)
        throttle = self.esc.throttle(u_mot, i_mot, self.battery.u_bat)
        i_esc = throttle * i_mot
        i_bat = i_esc * self.arms + 1
        t = self.battery.endurance(i_bat)
        d_max = 60 * v_max * t

        print('In forward flight mode:')
        print(f'\t- the maximum load is {max_load/G:.2f}kg')
        print(f'\t- the maximum pitch angle is {np.degrees(max_pitch):.2f}°')
        print(f'\t- the maximum forward speed is {v_max:.2f} m/s')
        print(f'\t- the maximum traveled distance is {d_max:.1f} m')


class allocation_controller:
    def __init__(self, arms, arms_angle, d, ct, cm):
        self.arms = arms
        fd_row = np.empty(arms)
        tx_row = np.empty(arms)
        ty_row = np.empty(arms)
        tz_row = np.empty(arms)

        fd_row.fill(ct)

        angle = arms_angle
        for tx in np.nditer(tx_row, op_flags=['readwrite']):
            tx = d * ct * np.sin(angle)
            angle = angle + np.pi * 2 / arms

        angle = arms_angle
        for ty in np.nditer(ty_row, op_flags=['readwrite']):
            ty = d * ct * np.cos(angle)
            angle = angle + np.pi * 2 / arms

        sign = 1
        for tz in np.nditer(tz_row, op_flags=['readwrite']):
            tz = cm * sign
            sign = sign * -1

        self.control_matrix = np.array(
            [fd_row,
             tx_row,
             ty_row,
             tz_row])

        self.alloc_matrix = np.linalg.inv(self.control_matrix)

    def allocate(self, set_points):
        return np.dot(self.control_matrix, set_points)


def main():
    drosix = toml.load('drosix.toml')
    propeller = Propeller(drosix['propeller']['diameter'],
                          drosix['propeller']['pitch'],
                          drosix['propeller']['blade'])
    motor = Motor(drosix['motor']['kv'],
                  drosix['motor']['i_max'],
                  drosix['motor']['i_nom'],
                  drosix['motor']['u_nom'],
                  drosix['motor']['r'])
    esc = ESC(drosix['esc']['r'])
    battery = Battery(drosix['battery']['capacity'],
                      drosix['battery']['cell'],
                      drosix['battery']['r'],
                      drosix['battery']['C'])
    drone = Drone(propeller, motor, esc, battery, 4, F_MASS)

    drone.hovering()
    drone.full_throtlle()
    drone.forward_flight()

    cr = (drone.w_max - drone.w_hover) / (1 - drone.throttle_hover)
    wb = drone.w_max - cr
    type(cr)

    drosix['motor']['cr'] = float(cr)
    drosix['motor']['wb'] = float(wb)
    drosix['propeller']['cm'] = drone.propeller.Cm / constants.RPM_2_RADPS**2
    drosix['propeller']['ct'] = drone.propeller.Ct / constants.RPM_2_RADPS**2
    drosix['hover'] = {
        'throttle': drone.throttle_hover,
        'w': drone.w_hover
    }

    encoder = toml.TomlNumpyEncoder()
    with open('drosix_model.toml', 'w') as f:
        toml.dump(drosix, f, encoder)


if __name__ == '__main__':
    main()
