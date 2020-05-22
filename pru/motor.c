#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_iep.h>
#include "util.h"
#pragma RESET_MISRA("all")

#define PERIOD_NS 20000000U

void main(void);
void configure_timer(void);

void main(void) {
    uint8_t run = 1U;

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    send_event(MST_4);

    __delay_cycles(5U);

    configure_timer();

    while(run == 1U) {
        switch(check_event1()) {
        case IEP_TIMER:
			CT_IEP.TMR_CMP_STS = 0x1U;
            send_event(MST_15);
            break;
        /* STOP */
        case MST_0:
            run = 0U;
            break;
        /* New data */
        case MST_5:
            /* send_event(MST_15); */
            /* handle new data */
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
    }

    send_event(MST_4);

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
