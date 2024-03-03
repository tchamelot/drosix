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

// FIXME create custom wraper for shared memory copying, memcpy is not safe
// TODO use dedicated function for rate controller

void main(void);
void configure_timer(void);
void set_armed(void);
void clear_armed(void);


void main(void) {
    uint32_t i;
    uint8_t run = 1U;
    struct pid_controller_t pids[7];
    odometry_t odometry;
    angles_t rate_set_point;
    angles_t rate_command;
    int32_t thrust = 0;

    /* performance */
    uint32_t cycle = 0U;
    uint32_t stall = 0U;


    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    /* wait motor to be ready */
    while(check_event0() != EVT_MOTOR_STATUS) {}
    /* send_event(MST_15); */
    send_event(EVT_CONTROLLER_STATUS);

    /* store pid coef in local memory */
    pid_init(&pids[0], controller.attitude_pid.roll.numerator, controller.attitude_pid.roll.denominator);
    pid_init(&pids[1], controller.attitude_pid.pitch.numerator, controller.attitude_pid.pitch.denominator);
    pid_init(&pids[2], controller.attitude_pid.yaw.numerator, controller.attitude_pid.yaw.denominator);
    pid_init(&pids[3], controller.thrust_pid.numerator, controller.thrust_pid.denominator);
    pid_init(&pids[4], controller.rate_pid.roll.numerator, controller.rate_pid.roll.denominator);
    pid_init(&pids[5], controller.rate_pid.pitch.numerator, controller.rate_pid.pitch.denominator);
    pid_init(&pids[6], controller.rate_pid.yaw.numerator, controller.rate_pid.yaw.denominator);

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
            rate_set_point.roll = pid_run(&pids[0], odometry.attitude.roll);
            rate_set_point.pitch = pid_run(&pids[1], odometry.attitude.pitch);
            rate_set_point.yaw = pid_run(&pids[2], odometry.attitude.yaw);
            thrust = pid_run(&pids[3], odometry.thrust);
            rate_command.roll = pid_run(&pids[4], rate_set_point.roll - odometry.rate.roll);
            rate_command.pitch = pid_run(&pids[5], rate_set_point.pitch - odometry.rate.pitch);
            rate_command.yaw = pid_run(&pids[6], rate_set_point.yaw - odometry.rate.yaw);

#pragma CHECK_MISRA("-10.3, -12.1")
            controller.pid_output[0] = (uint32_t)(199999 + (int32_t)(thrust + rate_command.roll + rate_command.pitch + rate_command.yaw));
            controller.pid_output[1] = (uint32_t)(199999 + (int32_t)(thrust - rate_command.roll + rate_command.pitch - rate_command.yaw));
            controller.pid_output[2] = (uint32_t)(199999 + (int32_t)(thrust + rate_command.roll - rate_command.pitch - rate_command.yaw));
            controller.pid_output[3] = (uint32_t)(199999 + (int32_t)(thrust - rate_command.roll - rate_command.pitch + rate_command.yaw));
#pragma RESET_MISRA("10.3, 12.1")
#pragma CHECK_MISRA("-11.3")
            controller.cycle = PRU0_CTRL.CYCLE - cycle;
            controller.stall = PRU0_CTRL.STALL - stall;
#pragma RESET_MISRA("11.3")

            send_event(EVT_PID_OUTPUT);

            if(controller.debug_config == PidLoop) {
              memcpy((void*)&controller.p_pid, (void*)&rate_set_point, sizeof(angles_t));
              memcpy((void*)&controller.v_pid, (void*)&rate_command, sizeof(angles_t));
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
            memcpy((void*)&odometry, (void*)&controller.pid_input, sizeof(odometry_t));
#pragma CHECK_MISRA("-11.3")
            controller.cycle = PRU0_CTRL.CYCLE - cycle;
            controller.stall = PRU0_CTRL.STALL - stall;
#pragma CHECK_MISRA("+11.3")
            if(controller.debug_config == PidNewData) {
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
    controller.pid_output[0] = 179999u;                /* Load motor arming value              */
    controller.pid_output[1] = 179999u;                /* Load motor arming value              */
    controller.pid_output[2] = 179999u;                /* Load motor arming value              */
    controller.pid_output[3] = 179999u;                /* Load motor arming value              */
    send_event(EVT_PID_OUTPUT);                     /* Commit motor arming values           */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear ECAP interrput flags           */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter                    */
    CT_INTC.SICR_bit.STS_CLR_IDX = ECAP_TIMER;      /* Clear PRU interrput flag             */
}
