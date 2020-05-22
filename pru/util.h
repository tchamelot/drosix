#ifndef __CONTROLER_H__
#define __CONTROLER_H__
#pragma RESET_MISRA("all")
#pragma CHECK_MISRA("-8.5")

/* PRU io registers */
volatile register uint32_t __R30;
volatile register uint32_t __R31;

/* PRU interrupt abstraction */
enum system_event_t {
    IEP_TIMER   = 7U,
    ECAP_TIMER  = 15U,
    MST_0       = 16U,
    MST_1       = 17U,
    MST_2       = 18U,
    MST_3       = 19U,
    MST_4       = 20U,
    MST_5       = 21U,
    MST_6       = 22U,
    MST_7       = 23U,
    MST_8       = 24U,
    MST_9       = 25U,
    MST_10      = 26U,
    MST_11      = 27U,
    MST_12      = 28U,
    MST_13      = 29U,
    MST_14      = 30U,
    MST_15      = 31U,
    /* Fallback */
    None        = 64U
};

inline void send_event(enum system_event_t evt);
inline enum system_event_t check_event0(void);
inline enum system_event_t check_event1(void);

inline void send_event(enum system_event_t evt) {
    if((evt <= MST_15) && (evt >= MST_0)) {
        __R31 = (uint32_t)0x20U | ((uint32_t)evt & 0x0000000FU);
    }
}

inline enum system_event_t check_event0(void) {
    enum system_event_t active_event0 = None;

    if((__R31 & 0x40000000U) != 0U) {
        active_event0 = (enum system_event_t)CT_INTC.HIPIR0;
    }

    /* clear the return interrput */
    CT_INTC.SICR_bit.STS_CLR_IDX = active_event0;

    return active_event0;
}

inline enum system_event_t check_event1(void) {
    enum system_event_t active_event1 = None;

    if((__R31 & 0x80000000U) != 0U) {
        active_event1 = (enum system_event_t)CT_INTC.HIPIR1;
    }

    /* clear the return interrput */
    CT_INTC.SICR_bit.STS_CLR_IDX = active_event1;

    return active_event1;
}
/* PRU ECAP abstraction */
#define ECAP_APWM_MODE  ((uint16_t)0x200U)
#define ECAP_CTRRUN     ((uint16_t)0x10U)
#define ECAP_INT_CMPEQ  ((uint16_t)0x80U)

#endif
