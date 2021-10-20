#ifndef __DROSIX_H__
#define __DROSIX_H__
#include <stdint.h>
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
    float a[3];
    float b[2];
};

struct controller_t {
    float inputs[7];
    uint32_t outputs[4];
    struct pid_parameter_t parameter[7];
    uint32_t debug_location;
    float p_pid[3];
    float v_pid[3];
    uint32_t cycle;
    uint32_t stall;
};

#define DEBUG_PID_LOOP      (1 << 0)
#define DEBUG_PID_NEW_DATA  (1 << 1)
#define DEBUG_PWM_STEP      (1 << 2)

#pragma DATA_SECTION(controller, ".sdata")
volatile far struct controller_t controller;

#pragma CHECK_MISRA("none")
#endif
