# PID

Drosix uses a basic proportional-integral-derivative controller ([PID](https://en.wikipedia.org/wiki/PID_controller)) to control its angular velocities and angular position.
A PID controller is mathematically expressed as (in the Laplace domain):

```math
G(s) = K_p.(1 + \frac{1}{T_i.s} + \frac{T_d.s}{\frac{T_d}{N}.s + 1}) \\\\
G(s) = \frac{\frac{K_p.T_d.T_i.(N+1)}{T_i.N}.s^2 + \frac{K_p.(T_d+Ti.N)}{T_i.N}.s + \frac{K_p}{T_i}}{\frac{T_d}{N}.s^2 + s} \\\\
G(s) = \frac{a_2.s^2 + a_1.s + a_0}{b_2.s^2 + s}
``` 

In order to implement a PID controller on a microcontroller, it is required to express in the discrete domain.
Passing from the continuous-time (Laplace) domain to the discrete-time domain is done through the [bilinear transform](https://en.wikipedia.org/wiki/Bilinear_transform).

```math
z = \frac{1 + s.\frac{T}{2}}{1 - s.\frac{T}{2}} \\\\
s = \frac{2}{T} . \frac{z - 1}{z + 1}
```

The PID's transfer function in the discrete domain is

```math
G(z) = K_p.(1 + \frac{1}{T_i.\frac{2}{T} . \frac{z - 1}{z + 1}} + \frac{T_d.\frac{2}{T} . \frac{z - 1}{z + 1}}{\frac{T_d}{N}.\frac{2}{T} . \frac{z - 1}{z + 1} + 1}) \\\\
G(z) = \frac{A_2.z^2 + A_1.z + A_0}{z^2 + B_1.z + B_0}
```
where 
```math
\begin{cases}
A_2 = (4.\frac{K_p.T_d.T_i.(N+1)}{T_i.N}+2.T.\frac{K_p.(T_d+Ti.N)}{T_i.N}+T^2.\frac{K_p}{T_i}) / (4. \frac{T_d}{N}+2.T)\\\\
A_1 = (2.T^2.\frac{K_p}{T_i} - 8.\frac{K_p.T_d.T_i.(N+1)}{T_i.N}) / (4. \frac{T_d}{N}+2.T) \\\\
A_0 = (4.\frac{K_p.T_d.T_i.(N+1)}{T_i.N}-2.T.\frac{K_p.(T_d+Ti.N)}{T_i.N}+T^2.\frac{K_p}{T_i}) / (4. \frac{T_d}{N}+2.T) \\\\
B_1 = (-8.\frac{T_d}{N}) / (4. \frac{T_d}{N}+2.T) \\\\
B_0 = (4.\frac{T_d}{N}-2.T) / (4. \frac{T_d}{N}+2.T)
\end{cases}
```
