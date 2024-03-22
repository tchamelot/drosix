# Mathematical model

## Notations

- $`T`$: Thrust in N
- $`M`$: Moment in N.m
- $`\sigma`$: Throttle $`[0;1]`$
- $`\Omega_M`$: Motor speed in rad/s
- $`N`$: Propeller speed in RPM
- $`U`$: Voltage in V
- $`\omega`$: Quadcopter angular velocity in rad/s
- $`\theta`$: Quadcopter angular position in rad
- $`J`$: Moment of inertia in kg.m2

## Laplace transform and differential equation

Considering a function $`f(t)`$, the following equation is used to go from Laplace to differential equation:

```math
f'(s) = s.f(s) - f(0) \rArr \frac{\partial f}{dt} = s.f - f(0) 
```



## Propulsion

A quadcopter has four propulsion systems.
A propulsion system is composed of a motor and a propeller mounted on it.
This model considers a propulsion system using a BDLC motor along with a two blades propeller.
Each propulsion system produces a thrust and a moment such as:

```math
\begin{aligned}
T = C_T \rho (\frac{N}{60})^2 D_{p}^4 \\\\
M = C_M \rho (\frac{N}{60})^2 D_{p}^5
\end{aligned}
```

Where $`D_p`$ is the propeller diameter in m and $`C_M`$ and $`C_T`$ are the thrust and moment coefficients.
$`C_M`$ and $`C_T`$ can be approximated using the [flyeval](flyeval.com) website.

The propeller and the motor spin at the same speeds.
In the case of a BLDC motor, an electronic speed controller (ESC) regulates the motor speed.
The ESC controls the DC equivalent voltage applied to the motor between \[0-$`U_{bat}`$\]V depending on the input throttle such as:

```math
U_{ESC} = \sigma U_{bat}
```

BDLS motors transfer function is second order with a mechanical mode and an electrical mode.
This model simplifies the electrical mode because it is much faster than the mechanical mode.
The simplified transfer function for a BDLC motor is:

```math
\Omega_M = \frac{K_M}{T_M S + 1} \sigma
```

It is necessary to model that ESCs have a dead zone where $`\sigma \neq 0`$ still provides $`V_{DC} = 0`$ V.
For Drosix's current ESCs, the dead zone ends at $`\sigma = 0.05`$.
The relation between $`\sigma`$ and $`\Omega_M`$ can be adjusted:

```math
\Omega_M = \frac{1}{T_M S + 1} (C_R \sigma + \omega_b) \\\\
```

Where $`C_r`$ and $`\omega_b`$ are the linear function coefficient binding the throttle to the motor speed (ignoring the mechanical mode).

The associated differential equation for simulation is:

```math
\frac{\partial \Omega_M(t)}{dt} = \frac{(C_R \sigma(t) + \omega_b) - \Omega_M(t)}{T_M}
```

## Body model

The body model relies on the following assumptions:
- the body is rigid;
- the mass and the moment of inertia of the body constant over the time;
- the centre of gravity of drone is the same as the geometrical centre of the frame;
- the only forces are the propulsion system thrust and the gravity forces are applied to the body.

![Drosix frame representation](/images/Kinematic.png)

The drone attitude is the angular positions and velocities of the frame.
Newton's second law of motion for rotation is:

```math
\sum{M_{\Delta i}} = J \dot{\omega}
```

$`M_{\Delta i}`$ are the moments of the forces applied to the drone against each axis.
Supposing the moment of inertia matrix is
```math
J = 
\begin{bmatrix}
    J_x & 0   & 0   \\\\
    0   & J_y & 0   \\\\
    0   & 0   & J_z
\end{bmatrix}
```

Then, the relation linking the attitude velocity to the propellers thrust and moment are:

```math
\begin{aligned}
    J_x \dot{\omega_x} = d (T_1 - T_2 - T_3 + T_4) \\\\
    J_y \dot{\omega_y} = d (T_1 + T_2 - T_3 - T_4) \\\\
    J_z \dot{\omega_z} = M_1 - M_2 + M_3 - M_4
\end{aligned}
```
with $`d`$ the distance between the motor and the center of the frame, $`d = \frac{\sqrt{2}}{2} L`$.
The motor speed can be injected in the previous equations such as:

```math
\begin{bmatrix}
    J_x & 0   & 0   \\\\
    0   & J_y & 0   \\\\
    0   & 0   & J_z
\end{bmatrix}
\begin{bmatrix}
    \dot{\omega_x} \\\\
    \dot{\omega_y} \\\\
    \dot{\omega_z}
\end{bmatrix} =
\begin{bmatrix}
    dC_T & -dC_T & -dC_T & dC_T  \\\\
    dC_T & dC_T  & -dC_T & -dC_T \\\\
    C_M  & -C_M  & C_M   & -C_M
\end{bmatrix}
\begin{bmatrix}
    \omega_1^2 \\\\
    \omega_2^2 \\\\
    \omega_3^2 \\\\
    \omega_4^2
\end{bmatrix}
```

From the [Propulsion][mathematical-model/#propulsion] section, the thrust along the $`z`$ axis of the frame is (with all the fixed parameters hidden behind $`C_T`$):

```math
f =
\begin{bmatrix}
    C_T & C_T & C_T & C_T
\end{bmatrix}
\begin{bmatrix}
    \omega_1^2 \\\\
    \omega_2^2 \\\\
    \omega_3^2 \\\\
    \omega_4^2
\end{bmatrix}
```

Expriming the system under the space state representation:

```math
\begin{aligned}
\dot{x} = A x + B u \\\\
y = C x + D u
\end{aligned}
```

The angular position is $`\dot{\theta} = \omega`$.

```math
\begin{bmatrix}
    \dot{\theta_x} \\\\
    \dot{\theta_y} \\\\
    \dot{\theta_z} \\\\
    \dot{\omega_x} \\\\
    \dot{\omega_y} \\\\
    \dot{\omega_z}
\end{bmatrix} =
\begin{bmatrix}
    0 & 0 & 0 & 1 & 0 & 0 \\\\
    0 & 0 & 0 & 0 & 1 & 0 \\\\
    0 & 0 & 0 & 0 & 0 & 1 \\\\
    0 & 0 & 0 & 0 & 0 & 0 \\\\
    0 & 0 & 0 & 0 & 0 & 0 \\\\
    0 & 0 & 0 & 0 & 0 & 0 \\\\
\end{bmatrix}
\begin{bmatrix}
    \theta_x \\\\
    \theta_y \\\\
    \theta_z \\\\
    \omega_x \\\\
    \omega_y \\\\
    \omega_z
\end{bmatrix} + 
\begin{bmatrix}
    0       & 0         & 0         & 0         \\\\
    0       & 0         & 0         & 0         \\\\
    0       & 0         & 0         & 0         \\\\
    \frac{dC_T}{J_x} & -\frac{dC_T}{J_x} & -\frac{dC_T}{J_x} &  \frac{dC_T}{J_x}  \\\\
    \frac{dC_T}{J_y} &  \frac{dC_T}{J_y} & -\frac{dC_T}{J_y} & -\frac{dC_T}{J_y}  \\\\
    \frac{C_M}{J_z}  & -\frac{C_M}{J_z}  &  \frac{C_M}{J_z}  & -\frac{C_M}{J_z}
\end{bmatrix}
\begin{bmatrix}
    \omega_1^2 \\\\
    \omega_2^2 \\\\
    \omega_3^2 \\\\
    \omega_4^2 \\\\
\end{bmatrix}
```

