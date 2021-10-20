#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <string.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "util.h"
#include "drosix.h"
#include "pid.h"
#pragma RESET_MISRA("all")

void main(void);
void configure_timer(void);
void set_armed(void);
void clear_armed(void);


void main(void) {
    uint32_t i;
    uint8_t run = 1U;
    struct pid_t pids[7];
    float p_error[3];
    float v_measure[3];
    float v_setpoint[3];
    float v_command[3];
    int32_t thrust = 0;

    /* performance */
    uint32_t cycle = 0U;
    uint32_t stall = 0U;

    memset(p_error, 0, sizeof(p_error));
    memset(v_measure, 0, sizeof(v_measure));
    memset(v_setpoint, 0, sizeof(v_setpoint));
    memset(v_command, 0, sizeof(v_command));

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    /* wait motor to be ready */
    while(check_event0() != EVT_MOTOR_STATUS) {}
    /* send_event(MST_15); */
    send_event(EVT_CONTROLLER_STATUS);

    /* store pid coef in local memory */
    for(i = 0u; i < 7u; i++) {
        pid_init(&pids[i], controller.parameter[i].a, controller.parameter[i].b);
    }

    configure_timer();

#pragma CHECK_MISRA("-11.3")
    PRU0_CTRL.CTRL_bit.CTR_EN = 1U;
#pragma RESET_MISRA("11.3")

    while(run == 1U) {
        switch(check_event0()) {
        /* PID */
        case EVT_PID_STEP:
            CT_ECAP.ECCLR = 0xffU;
#pragma CHECK_MISRA("-11.3")
            cycle = PRU0_CTRL.CYCLE;
            stall = PRU0_CTRL.STALL;
#pragma RESET_MISRA("11.3")
            v_setpoint[0] = pid_run(&pids[0], p_error[0]);
            v_setpoint[1] = pid_run(&pids[1], p_error[1]);
            v_setpoint[2] = pid_run(&pids[2], p_error[2]);
            /* thrust = pid_run(&pids[3]); */
            v_command[0] = pid_run(&pids[4], v_setpoint[0] - v_measure[0]);
            v_command[1] = pid_run(&pids[5], v_setpoint[1] - v_measure[1]);
            v_command[2] = pid_run(&pids[6], v_setpoint[2] - v_measure[2]);

            // TODO
#pragma CHECK_MISRA("-10.3, -12.1")
            controller.outputs[0] = (uint32_t)(199999 + (int32_t)(thrust + v_command[0] + v_command[1] + v_command[2]));
            controller.outputs[1] = (uint32_t)(199999 + (int32_t)(thrust - v_command[0] + v_command[1] - v_command[2]));
            controller.outputs[2] = (uint32_t)(199999 + (int32_t)(thrust + v_command[0] - v_command[1] - v_command[2]));
            controller.outputs[3] = (uint32_t)(199999 + (int32_t)(thrust - v_command[0] - v_command[1] + v_command[2]));
#pragma RESET_MISRA("10.3, 12.1")
#pragma CHECK_MISRA("-11.3")
            controller.cycle = PRU0_CTRL.CYCLE - cycle;
            controller.stall = PRU0_CTRL.STALL - stall;
#pragma RESET_MISRA("11.3")

            send_event(EVT_PID_OUTPUT);
            send_event(EVT_DEBUG);
            if(controller.debug_location & DEBUG_PID_LOOP) {
              controller.p_pid[0] =  v_setpoint[0];
              controller.p_pid[1] =  v_setpoint[1];
              controller.p_pid[2] =  v_setpoint[2];
              controller.v_pid[0] =  v_command[0];
              controller.v_pid[1] =  v_command[1];
              controller.v_pid[2] =  v_command[2];
              send_event(EVT_DEBUG);
            }
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
#pragma CHECK_MISRA("-11.3")
            cycle = PRU0_CTRL.CYCLE;
            stall = PRU0_CTRL.STALL;
#pragma CHECK_MISRA("+11.3")
            p_error[0] = controller.inputs[0];
            p_error[1] = controller.inputs[1];
            p_error[2] = controller.inputs[2];
            thrust = controller.inputs[3];
            v_measure[0] = controller.inputs[4];
            v_measure[1] = controller.inputs[5];
            v_measure[2] = controller.inputs[6];
#pragma CHECK_MISRA("-11.3")
            controller.cycle = PRU0_CTRL.CYCLE - cycle;
            controller.stall = PRU0_CTRL.STALL - stall;
#pragma CHECK_MISRA("+11.3")
            if(controller.debug_location & DEBUG_PID_NEW_DATA) {
              send_event(EVT_DEBUG);
            }
            break;
        case EVT_SET_ARMED:
            set_armed();
            break;
        case EVT_CLEAR_ARMED:
            clear_armed();
            for(i = 0u; i < 7; i++) {
              pid_reset(&pids[i]);
            }
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

void configure_timer(void) {
    CT_INTC.CMR3_bit.CH_MAP_15 = 0U;                /* Map S15 to channel 0                 */
    CT_INTC.EISR = ECAP_TIMER;                      /* Enable S15                           */
    CT_ECAP.CAP3 = (uint32_t)PID_PERIOD / 5U - 1U;  /* Set the sampling period              */
    CT_ECAP.ECCTL2 = ECAP_APWM_MODE | ECAP_CTRRUN;  /* APWM mode and counter free-running   */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter                    */
    CT_ECAP.ECEINT = 0U;                            /* Disable ECAP intterupt               */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear interrput flags                */
}

void set_armed(void) {
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter                    */
    CT_ECAP.ECEINT = ECAP_INT_CMPEQ;                /* Enable intterupt on CAP3 == TSCTR    */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear interrput flags                */
}

void clear_armed(void) {
    CT_ECAP.ECEINT = 0u;                            /* Disable ECAP interupt                */
    controller.outputs[0] = 179999u;                /* Load motor arming value              */
    controller.outputs[1] = 179999u;                /* Load motor arming value              */
    controller.outputs[2] = 179999u;                /* Load motor arming value              */
    controller.outputs[3] = 179999u;                /* Load motor arming value              */
    send_event(EVT_PID_OUTPUT);                     /* Commit motor arming values           */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear ECAP interrput flags           */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter                    */
    CT_INTC.SICR_bit.STS_CLR_IDX = ECAP_TIMER;      /* Clear PRU interrput flag             */
}
