#pragma CHECK_MISRA("none")
#include <stdint.h>
#include <am335x/pru_cfg.h>
#include <am335x/pru_intc.h>
#include <am335x/pru_ecap.h>
#include "controller.h"
#pragma RESET_MISRA("all")

#define PERIOD_NS 10000000U

volatile register uint32_t __R30;
volatile register uint32_t __R31;

struct pid_t {
    int32_t kp;
    int32_t ki;
    int32_t kd;
};

struct controller_t {
    volatile int32_t input;
    volatile int32_t output;
    volatile struct pid_t coef;
};

#pragma DATA_SECTION(controller, ".sdata")
volatile far struct controller_t controller;

void configure_timer(void);
void main(void);

void configure_timer(void) {
    CT_INTC.GER_bit.EN_HINT_ANY = 1U;    /* Enable global interrupt */
    CT_INTC.CMR3_bit.CH_MAP_15 = 0U;     /* Map S15 to channel 0 */
    CT_INTC.ESR0 |= (uint32_t)1U << 15;            /* Enable S15 */
    CT_ECAP.CAP3 = (uint32_t)PERIOD_NS / 5U - 1U;   /* Set the sampling period */
    CT_ECAP.ECCTL2 = (uint16_t)(((uint32_t)1U << 9) | ((uint32_t)1U << 4)); /* APWM mode and counter free-running */
    CT_ECAP.TSCTR = 0U;                  /* Reset the counter */
    CT_ECAP.ECEINT = 0x80U;              /* Enable intterupt on CAP3 == TSCTR */
    CT_ECAP.ECCLR  = 0xffU;              /* Clear interrput flags */
    CT_INTC.SECR0 = 0xFFFFFFFFU;         /* Clear the status of all interrupts */
	CT_INTC.SECR1 = 0xFFFFFFFFU;
}

void main(void) {
    volatile uint32_t i;
    int32_t err = 0;
    int32_t diff = 0;
    int32_t previous = 0;
    struct pid_t pid;

    CT_CFG.SYSCFG_bit.STANDBY_INIT = 0U;    /* enable OCP master port */

    /* notify host */
    __R31 = 0x23U;

    /* wait for host */
    while((__R31 >> 30) & 1U == 0U) {}
    CT_INTC.SICR_bit.STS_CLR_IDX = 21U;

    /* store pid coef in local memory */
    pid.kp = controller.coef.kp;
    pid.ki = controller.coef.ki;
    pid.kd = controller.coef.kd;

    for(i = 0U; i < 8000000U; i++) {
    }

    configure_timer();
    /* wait timer interrupt */
    for(i = 0U; i < 10U; i++) {
        while((__R31 >> 30) & 1U == 0U) {}
        CT_INTC.SICR_bit.STS_CLR_IDX = 15U;  /* Clear PRU interrupt */
        CT_ECAP.ECCLR = 0xffU;               /* Clear timer interrupt */
    
        err += controller.input;
        diff = controller.input - previous;
        controller.output = (pid.kp * controller.input) +
                            (pid.kd * diff) +
                            (pid.ki * err);

        __R31 = 0x23U;                       /* Send interrupt S16 to host */
    }

    /* notify host */
    /* __R31 = 0x23; */
    
    /* wait for host */
    while((__R31 >> 30) & 1U == 0U) {}
    CT_INTC.SICR_bit.STS_CLR_IDX = 21U;
    for(i = 0U; i < 8000000U; i++) {
    }
    
    /* notify host */
    __R31 = 0x23U;

    __halt();
}
