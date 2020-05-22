#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_iep.h>
#include "util.h"
#include "drosix.h"
#pragma RESET_MISRA("all")

#define PERIOD_NS 20000000U

void main(void);
void configure_timer(void);

void main(void) {
    uint8_t run = 1U;
    uint32_t duty_cycles = 

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    send_event(EVT_MOTOR_STATUS);

    __delay_cycles(5U);

    configure_timer();

    while(run == 1U) {
        switch(check_event1()) {
        case EVT_PWM_STEP:
			CT_IEP.TMR_CMP_STS = 0x1U;
            send_event(MST_15);
            set_pins(ALL_MOTORS);
            break;
        /* STOP */
        case EVT_MOTOR_STOP:
            run = 0U;
            break;
        /* New data */
        case EVT_PID_OUTPUT:
            /* send_event(MST_15); */
            /* handle new data */
            duty_cycles = controller.outputs;
            break;
        /* No event yet */
        case None:
            break;
        /* Unexpected interrput */
        default:
            run = 0U;
            break;
        }

        /* process pwms */
        if(CT_IEP.TMR_CNT >= duty_cycles) {
            clear_pins(ALL_MOTORS);
        }
    }

    clear_pins(ALL_MOTORS);
    send_event(EVT_MOTOR_STATUS);

    __halt();
}

void configure_timer(void) {
    CT_INTC.CMR1_bit.CH_MAP_7 = 1U;                 /* Map S7 to channel 0 */
    CT_INTC.EISR = IEP_TIMER;                       /* Enable S15 */
	CT_IEP.TMR_GLB_CFG_bit.CNT_EN = 0U;             /* Disable counter */
	CT_IEP.TMR_CNT = 0xFFFFFFFFU;		            /* Reset Count register */
	CT_IEP.TMR_GLB_STS_bit.CNT_OVF = 0x1U;          /* Clear overflow status register */
	CT_IEP.TMR_CMP0 = PERIOD_NS / 5U - 1U;                     /* Set compare0 value */
	CT_IEP.TMR_CMP_STS_bit.CMP_HIT = 0xFFU;         /* Clear compare status */
	CT_IEP.TMR_COMPEN_bit.COMPEN_CNT = 0x0U;        /* Disable compensation */
	CT_IEP.TMR_CMP_CFG_bit.CMP0_RST_CNT_EN = 0x1U;  /* Disable CMP0 and reset on event */
	CT_IEP.TMR_CMP_CFG_bit.CMP_EN = 0x1U;
	CT_IEP.TMR_GLB_CFG_bit.DEFAULT_INC = 0x1U;		/* Configure incr value */
	CT_IEP.TMR_GLB_CFG_bit.CNT_EN = 1U;             /* Enable counter */
	/* CT_INTC.SECR0 = 0xFFFFFFFFU;
	CT_INTC.SECR1 = 0xFFFFFFFFU;*/
}
