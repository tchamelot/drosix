#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "util.h"
#include "pid.h"
#include "drosix.h"

void main(void) {
    uint32_t cycle = 0U;
    uint32_t stall = 0U;
    struct pid_t pid = {0.335, 4.12e-5, 1.67, 1, 0.f, 0.f, 0.f} ;

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */
    /* Notify ready */
    send_event(EVT_CONTROLLER_STATUS);

    /* Wait for start signal */
    while(check_event0() != EVT_PID_NEW_DATA) {}
    PRU0_CTRL.CTRL_bit.CTR_EN = 1U;
    
    cycle = PRU0_CTRL.CYCLE;
    stall = PRU0_CTRL.STALL;
    run_pid(&pid, 0.5);
    controller.pru0_cycle = PRU0_CTRL.CYCLE - cycle;
    controller.pru0_stall = PRU0_CTRL.STALL - stall;
    send_event(MST_15);
    __delay_cycles(10U);
    send_event(EVT_CONTROLLER_STATUS);
    __halt();
}
