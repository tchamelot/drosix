#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "util.h"
#pragma RESET_MISRA("all")

#define PERIOD_NS 10000000U

struct pid_parameter_t {
    int32_t kp;
    int32_t ki;
    int32_t kd;
};

struct pid_t {
    struct pid_parameter_t _parameter;
    int32_t error;
    int32_t input[2];
};

struct controller_t {
    volatile int32_t inputs;
    volatile int32_t outputs;
    volatile struct pid_parameter_t parameter;
    volatile uint32_t pru0_cycle;
    volatile uint32_t pru0_stall;
};

#pragma DATA_SECTION(controller, ".sdata")
volatile far struct controller_t controller;

void main(void);
int32_t run_pid(struct pid_t* pid);
void configure_timer(void);


void main(void) {
    uint8_t run = 1U;
    struct pid_t pids;

    /* performance */
    /* uint32_t cycle = 0U; */
    /* uint32_t stall = 0U; */

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    /* wait motor to be ready */
    while(check_event0() != MST_4) {}
    /* send_event(MST_15); */
    send_event(MST_3);

    /* store pid coef in local memory */
    pids._parameter = controller.parameter;

    configure_timer();

    while(run == 1U) {
        switch(check_event0()) {
        /* PID */
        case ECAP_TIMER:
            CT_ECAP.ECCLR = 0xffU;
            controller.outputs = run_pid(&pids);
            send_event(MST_5);
            /* send_event(MST_15); */
            break;
        /* STOP */
        case MST_1:
            send_event(MST_0);
            break;
        /* Motor stop or error */
        case MST_4:
            run = 0U;
            break;
        /* New data */
        case MST_2:
            /* handle new data */
            pids.input[0] = controller.inputs;
            break;
        /* No event yet */
        case None:
            break;
        /* Unexpected interrput */
        default:
            send_event(MST_0);
            break;
        }
    }

    send_event(MST_3);

    __halt();
}

int32_t run_pid(struct pid_t* pid) {
    int32_t delta, result;

    pid->error += pid->input[0];
    delta = pid->input[0] - pid->input[1];
    result = pid->_parameter.kp * pid->input[0];
    result += (pid->_parameter.ki * pid->error);
    result += (pid->_parameter.kd * delta);

    /* TODO handle min and max */

    return result;
}

void configure_timer(void) {
    CT_INTC.CMR3_bit.CH_MAP_15 = 0U;                /* Map S15 to channel 0 */
    CT_INTC.EISR = ECAP_TIMER;                      /* Enable S15 */
    CT_ECAP.CAP3 = (uint32_t)PERIOD_NS / 5U - 1U;   /* Set the sampling period */
    CT_ECAP.ECCTL2 = ECAP_APWM_MODE | ECAP_CTRRUN;  /* APWM mode and counter free-running */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter */
    CT_ECAP.ECEINT = ECAP_INT_CMPEQ;                /* Enable intterupt on CAP3 == TSCTR */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear interrput flags */
}
