#ifndef __DROSIX_H__
#define __DROSIX_H__
#include "util.h"
#pragma RESET_MISRA("all")

/* Drosix related event */
#define EVT_MOTOR_STOP          (MST_0)
#define EVT_CONTROLLER_STOP     (MST_1)
#define EVT_PID_NEW_DATA        (MST_2)
#define EVT_CONTROLLER_STATUS   (MST_3)
#define EVT_MOTOR_STATUS        (MST_4)
#define EVT_PID_OUTPUT          (MST_5)
#define EVT_SET_ARMED           (MST_6)
#define EVT_CLEAR_ARMED         (MST_7)
#define EVT_DEBUG               (MST_15)
#define EVT_PID_STEP            (ECAP_TIMER)
#define EVT_PWM_STEP            (IEP_TIMER)

/* Event periods (nano seconds) */
#define PID_PERIOD 10000000U
#define PWM_PERIOD 10000000U

/* GPO mapping */

#define MOTOR_1 (GPO_5)
#define MOTOR_2 (GPO_4)
#define MOTOR_3 (GPO_7)
#define MOTOR_4 (GPO_6)
#define ALL_MOTORS (MOTOR_1 | MOTOR_2 | MOTOR_3 | MOTOR_4)

/* Data abstraction */
struct pid_parameter_t {
    int32_t kp;
    int32_t ki;
    int32_t kd;
};

// struct pid_t {
//     struct pid_parameter_t _parameter;
//     int32_t error;
//     int32_t input[2];
// };

struct controller_t {
    volatile int32_t inputs[7];
    volatile uint32_t outputs[4];
    volatile struct pid_parameter_t parameter[7];
    volatile uint32_t pru0_cycle;
    volatile uint32_t pru0_stall;
};

#pragma DATA_SECTION(controller, ".sdata")
volatile far struct controller_t controller;

#pragma CHECK_MISRA("none")
#endif
