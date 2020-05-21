#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_ctrl.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "controller.h"
#pragma RESET_MISRA("all")

#define PERIOD_NS 10000000U

struct pid_t {
    int32_t kp;
    int32_t ki;
    int32_t kd;
};

struct controller_t {
    volatile int32_t input;
    volatile int32_t output;
    volatile struct pid_t coef;
    volatile uint32_t pru0_cycle;
    volatile uint32_t pru0_stall;
};

#pragma DATA_SECTION(controller, ".sdata")
volatile far struct controller_t controller;

void configure_timer(void);
void main(void);

void configure_timer(void) {
    CT_INTC.CMR3_bit.CH_MAP_15 = 0U;                /* Map S15 to channel 0 */
    CT_INTC.EISR = PRU_INT_ECAP;                    /* Enable S15 */
    CT_ECAP.CAP3 = (uint32_t)PERIOD_NS / 5U - 1U;   /* Set the sampling period */
    CT_ECAP.ECCTL2 = ECAP_APWM_MODE | ECAP_CTRRUN;  /* APWM mode and counter free-running */
    CT_ECAP.TSCTR = 0U;                             /* Reset the counter */
    CT_ECAP.ECEINT = ECAP_INT_CMPEQ;                /* Enable intterupt on CAP3 == TSCTR */
    CT_ECAP.ECCLR  = 0xffU;                         /* Clear interrput flags */
    CT_INTC.SECR0 = 0xFFFFFFFFU;                    /* Clear the status of all interrupts */
	CT_INTC.SECR1 = 0xFFFFFFFFU;
}

void main(void) {
    volatile uint32_t i;
    int32_t err = 0;
    int32_t diff = 0;
    int32_t previous = 0;
    struct pid_t pid;

    /* performance */
    uint32_t cycle = 0U;
    uint32_t stall = 0U;

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */
    CT_INTC.SECR0 = 0xFFFFFFFFU;            /* Clear the status of all interrupts */
	CT_INTC.SECR1 = 0xFFFFFFFFU;

    notify_mst(MST_3);

    wait_for_int_host0(PRU_INT_MST_5);

    /* store pid coef in local memory */
    pid.kp = controller.coef.kp;
    pid.ki = controller.coef.ki;
    pid.kd = controller.coef.kd;

    /* for(i = 0U; i < 8000000U; i++) {
    } */

    configure_timer();

    /* enable cycle counter */
    #pragma CHECK_MISRA("none")
    PRU0_CTRL.CTRL_bit.CTR_EN = 1U;
    #pragma RESET_MISRA("all")

    /* wait timer interrupt */
    for(i = 0U; i < 10U; i++) {
        wait_for_int_host0(PRU_INT_ECAP);
        CT_ECAP.ECCLR = 0xffU;                          /* Clear timer interrupt */
    
        #pragma CHECK_MISRA("none")
        cycle = PRU0_CTRL.CYCLE;
        stall = PRU0_CTRL.STALL;
        #pragma RESET_MISRA("all")

        err += controller.input;
        diff = controller.input - previous;
        controller.output = (pid.kp * controller.input) +
                            (pid.kd * diff) +
                            (pid.ki * err);

        notify_mst(MST_4);

        #pragma CHECK_MISRA("none")
        controller.pru0_stall = PRU0_CTRL.STALL - stall;
        controller.pru0_cycle = PRU0_CTRL.CYCLE - cycle;
        #pragma RESET_MISRA("all")
    }

    wait_for_int_host0(PRU_INT_MST_5);
    /* for(i = 0U; i < 8000000U; i++) {
    } */
    
    /* notify host */
    notify_mst(MST_3);

    __halt();
}


inline void notify_mst(enum mst_interrupt_t evt) {
    __R31 = (uint32_t)0x20U | evt;
}

inline void wait_for_int_host0(uint32_t evt) {
    do{
        while((__R31 & 0x40000000U) == 0U) {}
    }while(CT_INTC.HIPIR0 != evt);
    CT_INTC.SICR_bit.STS_CLR_IDX = evt;
}
