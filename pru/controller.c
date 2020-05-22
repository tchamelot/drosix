#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "util.h"
#include "drosix.h"
#pragma RESET_MISRA("all")

void main(void);
uint32_t run_pid(struct pid_t* pid);
void configure_timer(void);


void main(void) {
    uint8_t run = 1U;
    struct pid_t pids;

    /* performance */
    /* uint32_t cycle = 0U; */
    /* uint32_t stall = 0U; */

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    /* wait motor to be ready */
    while(check_event0() != EVT_MOTOR_STATUS) {}
    /* send_event(MST_15); */
    send_event(EVT_CONTROLLER_STATUS);

    /* store pid coef in local memory */
    pids._parameter = controller.parameter;

    configure_timer();

    while(run == 1U) {
        switch(check_event0()) {
        /* PID */
        case EVT_PID_STEP:
            CT_ECAP.ECCLR = 0xffU;
            controller.outputs = run_pid(&pids);
            send_event(MST_5);
            /* send_event(MST_15); */
            break;
        /* STOP */
        case EVT_CONTROLLER_STOP:
            send_event(EVT_MOTOR_STOP);
            break;
        /* Motor stop or error */
        case EVT_MOTOR_STATUS:
            run = 0U;
            break;
        /* New data */
        case EVT_PID_NEW_DATA:
            /* handle new data */
            pids.input[0] = controller.inputs;
            break;
        /* No event yet */
        case None:
            break;
        /* Unexpected interrput */
        default:
            send_event(EVT_MOTOR_STOP);
            break;
        }
    }

    send_event(EVT_CONTROLLER_STATUS);

    __halt();
}

uint32_t run_pid(struct pid_t* pid) {
    int32_t delta, result;

    pid->error += pid->input[0];
    delta = pid->input[0] - pid->input[1];
    result = pid->_parameter.kp * pid->input[0];
    result += (pid->_parameter.ki * pid->error);
    result += (pid->_parameter.kd * delta);

    /* TODO handle min and max */
    if(result >= 399999) {
        result = 399999;
    }
    if(result <= 179999) {
        result = 179999;
    }

    return (uint32_t)result;
}

void configure_timer(void) {
    CT_INTC.CMR3_bit.CH_MAP_15 = 0U;                /* Map S15 to channel 0 */
    CT_INTC.EISR = ECAP_TIMER;                      /* Enable S15 */
    CT_ECAP.CAP3 = (uint32_t)PID_PERIOD / 5U - 1U;  /* Set the sampling period */
    CT_ECAP.ECCTL2 = ECAP_APWM_MODE | ECAP_CTRRUN;  /* APWM mode and counter free-running */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter */
    CT_ECAP.ECEINT = ECAP_INT_CMPEQ;                /* Enable intterupt on CAP3 == TSCTR */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear interrput flags */
}
