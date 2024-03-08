# PRU interfaces

The flight controller is responsible for the PRU subsystems.
The flight controller and the PRUs communicates through interrupt events and shared memory.




## Interrupt mapping

| Event             | From        | To    | Code        | Channel |
|:------------------|:------------|:------|:------------|:--------|
| MOTOR_STOP        | PRU0        | PRU1  | MST0 (S16)  | 1       |
| CONTROLLER_STOP   | HOST        | PRU0  | MST1 (S17)  | 0       |
| PID_NEW_DATA      | HOST        | PRU0  | MST2 (S18)  | 0       |
| CONTROLLER_STATUS | PRU0        | HOST0 | MST3 (S19)  | 2       |
| MOTOR_STATUS      | PRU1        | PRU0  | MST4 (S20)  | 0       |
| PID_OUTPUT        | PRU0        | PRU1  | MST5 (S21)  | 1       |
| SET_ARMED         | HOST        | PRU0  | MST6 (S22)  | 0       |
| CLEAR_ARMED       | HOST        | PRU0  | MST7 (S23)  | 0       |
| DEBUG             | PRU0 / PRU1 | HOST1 | MST15 (S31) | 3       |

## Shared memory layout

```mermaid
classDiagram
    class SharedMemory {
        +[float;7] pid_input
        +[uint32_t;4] pid_output
        +AnglePid attitude_pid
        +Pid thrust_pid
        +AnglePid rate_pid
        +DebugConfig debug_config
        +[float;3] p_pid
        +[float;3] v_pid
        +uint32_t cycle
        +uint32_t stall
    }

    class AnglePid {
        +Pid roll
        +Pid pitch
        +Pid yaw
    }

    class Pid {
        +[float;3] numerator
        +[float;2] denominator
    }

    class DebugConfig {
        << enumeration >>
        PidLoop
        PidNewData
        PwmStep
    }

    SharedMemory..>AnglePid
    SharedMemory..>DebugConfig
    SharedMemory..>Pid
    AnglePid..>Pid

```

- [ ] Use named fields instead of anonymous arrays

## PRU subsystems communication sequences

```mermaid
sequenceDiagram
    participant controller as Flight Controller
    participant pru0 as PID Controller
    participant pru1 as PWM Controller
    participant shmem as Shared Memory

    Note over controller,shmem: Start sequence

    controller->>shmem: Writes config
    controller->>+pru0: Starts
    controller->>+pru1: Starts
    pru1->>shmem: Reads config
    shmem-->>pru1: 
    pru1--)pru0: MOTOR_STATUS
    pru0->>shmem: Reads config
    shmem-->>pru0: 
    pru0--)controller: CONTROLLER_STATUS

    Note over controller,shmem: Nominal sequence
    controller-)pru0: SET_ARMED
    loop 100Hz
        controller->>shmem: Writes PID input
        controller-)pru0: PID_NEW_DATA
        pru0->>shmem: Reads PID input
        shmem-->>pru0: 
        pru0->>pru0: Computes PID
        pru0->>shmem: Writes PID output
        pru0-)pru1: PID_OUTPUT
        pru1->>shmem: Reads PID output
        shmem-->>pru1: 
        pru1->>pru1: Compute PWM
    end
    controller-)pru0: CLEAR_ARMED

    Note over controller,shmem: Stop Sequence
    opt Stop request
        controller-)pru0: CONTROLLER_STOP
        pru0-)pru1: MOTOR_STOP
    end
    pru1-)pru0: MOTOR_STATUS
    deactivate pru1
    pru0-)controller: CONTROLLER_STATUS
    deactivate pru0
```
