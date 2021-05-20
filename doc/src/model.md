# Drosix quadcopter model

In order to be able to control a quadcopter drone (or any UAV), the on-board
computer uses a controller build and tuned based on a mathematical model.

## Quadcopter model

![image](./Kinematic.png)

The quadcopter has 4 motors which are named `1`, `2`, `3` and `4`. `1` and `4`
spin clockwise. `2` and `3` spin anti clockwise. The distance between the center
of the drone and each motor is `L`. The yaw is the clockwise rotation around the
`z` axis. The roll is the clockwise rotation around the `x` axis. The pitch is
is the clockwise rotation around the `y` axis.

In order to determine the equation binding the roll, pitch and yaw to the motor
angular velocity, I used the rotational dynamics law.

\\[
\sum_{}^n M_{/O} = J_\Delta \frac{d\Omega}{dt}
\\]

There are 4 forces which produce a non null moment here, one for each motor. The
equations are:

\\( M_{\overrightarrow{F_1}/O} = \begin{pmatrix} LF_1 \\\\ LF_1 \\\\ 0 \\\\ \end{pmatrix} \\),
\\( M_{\overrightarrow{F_2}/O} = \begin{pmatrix} -LF_2 \\\\ LF_2 \\\\ 0 \\\\ \end{pmatrix} \\),
\\( M_{\overrightarrow{F_3}/O} = \begin{pmatrix} LF_3 \\\\ -LF_3 \\\\ 0 \\\\ \end{pmatrix} \\),
\\( M_{\overrightarrow{F_4}/O} = \begin{pmatrix} -LF_4 \\\\ -LF_4 \\\\ 0 \\\\ \end{pmatrix} \\)

Moreover, the 4 propellers produce 4 couples on the `z` axis such as:
\\( M_{propellers} = \begin{pmatrix} 0 \\\\ 0 \\\\ C_1 - C_2 - C_3 + C_4 \\\\ \end{pmatrix} \\)


Those equations can represent the roll, the pitch and the yaw.

\\[
\sum_{}^n M_{/O} = L\begin{pmatrix} F_1 - F_2 + F_3 - F_4 \\\\ F_1 + F_2 - F_3 - F_4 \\\\ C_1 - C_2 - C_3 + C_4 \\\\ \end{pmatrix} = J_\Delta \frac{d}{dt}\begin{pmatrix} roll \\\\ pitch \\\\ yaw \\\\ \end{pmatrix}
\\]

This equations has one unknown variable. \\(J_\Delta\\) represents the drone's
moment of inertia. It is the sum of each part's moment of inertia. For the sake
of simplicity, I will represent each motor block as a point, each arm and the
center block as full parallelepiped.

\\(J_{motor/\Delta} = M_{motor} L_{arm}^2 = \frac{L}{4} M_{motor} \\) with
\\(M_motor\\) being the sum of the masses of the motor and the propeller.

\\(J_{arm} = \frac{1}{12} M_{arm} (L_{arm}^2 + l_{arm}^2) \\) with `L` being the
length and `l` being the width.

\\(J_{center} = \frac{1}{12} M_{center} (L_{center}^2 + l_{center}^2) \\).

Finally

\\[
\begin{align}
J_\Delta & = 4J_{motor/\Delta} + 4J_{arm} + J_{center} \\\\ 
 & = LM_{motor} + \frac{1}{3} M_{arm} (L_{arm}^2 + l_{arm}^2)
 + \frac{1}{12} M_{arm}(L_{center}^2 + l_{center}^2) \\\\
\end{align}
\\]

## Propeller model

In the previous section, I presented the equations binding the roll, pitch and
yaw to the propellers' forces. Those forces are binded to the motor angular
velocity. I will use an approximation known as the Abbott formula:

\\[
\overrightarrow{F_{propeller}} = 28.35 \times 10^{-10} \times D^3 \times
P \times N^2
\\]

Where \\(\overrightarrow{F_{propeller}}\\) is the force in gf (\\(1gf
= 9.81mN\\)), \\(D\\) is the propeller diameter in inch, \\(P\\) is the
propeller pitch in inch and \\(N\\) is the angular velocity in rotation per
minute.

## Controller model

The controller is designed to make the quadcopter stable. Without it, we could
not pilot the drone. The controller is based on a control loop mechanism using
feedbacks. It means that the input of the controller is composed of both the
set-point and the state of the drone. The controller outputs 4 commands for each
motor.

Here, I talked about state like for a State-space representation. It is
a mathematical model representing a system composed of several inputs and
outputs. It is well suited for UAV applications however, it is beyond my
skills.

Here, the state of the drone will be built from the angular velocity measured
with a 3 axis gyroscope. Those angles are the roll, pitch and yaw discussed
previously. They will be the inputs of the controller. Now the problem is that
we have 3 inputs, 4 outputs and only 3 equations. To solve this system of
equation, I make the assumption that the drone always function around a fixed
set-point for each angle and that each equation is composed of only two
forces. One force will be the positive sum while the other will be the negative
sum.

\\[
\frac{d}{dt} \begin{pmatrix} roll \\\\ pitch \\\\ yaw \\\\ \end{pmatrix} = 
\frac{1}{J_\Delta} \begin{pmatrix}
    L (\Delta_{F_{13}} - \Delta{F_{24}}) \\\\
    L (\Delta_{F_{12}} - \Delta{F_{34}}) \\\\
    \Delta_{C_{14}} - \Delta{C_{23}} \\\\
\end{pmatrix} 
\\]

Each rotation is center on the point \\(O\\) so \\(\Delta_{F_{ab}}\\) is
equal to \\(-\Delta_{F_{cd}}\\) so that the propellers create a couple center
on \\(O\\) with only one component (either `x`, `y` or `z`). Moreover, each
forces \\(\Delta_{F_{ab}}\\) is equally reparteed on the two motors \\(a\\)
and \\(b\\). Then the controllers can output the new couples to set for each
angular velocity and fuse them together to compute each motor force.

\\(
\frac{d}{dt} \begin{pmatrix} roll \\\\ pitch \\\\ yaw \\\\ \end{pmatrix} = 
\frac{1}{J_\Delta} \begin{pmatrix}
    L \Delta_{roll} \\\\
    L \Delta_{pitch} \\\\
    \Delta_{yaw} \\\\
\end{pmatrix} 
\\)
\\(
\begin{align}
& F_{1} = \frac{1}{4} (\Delta_{roll} + \Delta_{pitch} + \Delta_{yaw}) \\\\
& F_{2} = \frac{1}{4} (- \Delta_{roll} + \Delta_{pitch} - \Delta_{yaw}) \\\\
& F_{3} = \frac{1}{4} (\Delta_{roll} - \Delta_{pitch} - \Delta_{yaw}) \\\\
& F_{4} = \frac{1}{4} (- \Delta_{roll} - \Delta_{pitch} + \Delta_{yaw}) \\\\
\end{align} 
\\)

## Model parameters

Generated using [flyeval](https://www.flyeval.com/).

* Mass = 0.9kg
* g = 9.8m/s²
* Inertia matrix = \\(\begin{pmatrix}
    1.334e^{-2} 0           0 \\\\
    0           1.334e^{-2} 0 \\\\
    0           0           2.557e^{-2}
    \end{pmatrix}\\) [kg.m²]
* Ct = \\(1.201e^{-5}\\) N/(rad/s²)
* Cm = \\(1.606e^{-7}\\) N.m/(rad/s²)
* Cr = 533.53 rad/s² (throttle [0-1] to motor steady speed)
* \\(\omega_{ss} = Cr \times \sigma + \omega_{b}\\) \\(\omega_{b}\\) = 119
  rad/s²
* Motor / propeller inertia Jm = \\(1.30e^{-4}\\) kg.m²
* Motor response time Tm = 0.0164s
* Air-Drag Coef. by Drag (N) dividing fly-speed² (m/s), i.e. (Cd=D/V²) Cd
  = \\(6.579^e{-2}\\) N/(m/s²)
* Air-Torque Coef. by Torque (N.m) dividing rotation-speed² (rad/s), i.e. (Cdm=M/w²) Cdm = 9.012e-3

## Evaluation

1. Hover mode: hover time, throttle, Ie, Ue, Ib, N
2. Max throttle: Ie, Ue, Ib, N, efficiency
3. Forward mode: max load, max pitch
4. Forward mode: max speed, max distance


