# Flight controller

The flight controller is spread between the two am335x's PRUs and the main
cortex processor. 

## Interrupt mapping

| Event             | From         | Code       | Channel | Host  |
|:------------------|:-------------|:-----------|:--------|:------|
| MOTOR STOP        | PRU0 (HOST)  | MST0 (S16)  | 1       | PRU1  |
| CONTROLLER STOP   | HOST         | MST1 (S17)  | 0       | PRU0  |
| PID NEW DATA      | HOST         | MST2 (S18)  | 0       | PRU0  |
| CONTROLLER STATUS | PRU0         | MST3 (S19)  | 2       | HOST0 |
| MOTOR STATUS      | PRU1         | MST4 (S20)  | 0       | PRU0  |
| PID OUTPUT        | PRU0         | MST5 (S21)  | 1       | PRU1  |
| SET ARMED         | HOST         | MST6 (S22)  | 0       | PRU0  |
| CLEAR ARMED       | HOST         | MST7 (S23)  | 0       | PRU0  |
| DEBUG             | PRU0 / PRU1  | MST15 (S31) | 3       | HOST1 |
