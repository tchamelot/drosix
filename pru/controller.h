#ifndef __CONTROLER_H__
#define __CONTROLER_H__
#pragma RESET_MISRA("all")

/* PRU CONSTANT */
#define PRU_INT_ECAP    15U
#define PRU_INT_MST_3   19U
#define PRU_INT_MST_4   20U  
#define PRU_INT_MST_5   21U  

#define ECAP_APWM_MODE  ((uint16_t)0x200U)
#define ECAP_CTRRUN     ((uint16_t)0x10U)

#define ECAP_INT_CMPEQ  ((uint16_t)0x80U)

/* PRU utilities */
volatile register uint32_t __R30;
volatile register uint32_t __R31;

enum mst_interrupt_t{
    MST_0 = 0U,
    MST_1 = 1U,
    MST_2 = 2U,
    MST_3 = 3U,
    MST_4 = 4U,
    MST_5 = 5U,
    MST_6 = 6U,
    MST_7 = 7U,
    MST_8 = 8U,
    MST_9 = 9U,
    MST_10 = 10U,
    MST_11 = 11U,
    MST_12 = 12U,
    MST_13 = 13U,
    MST_14 = 14U,
    MST_15 = 15U
};

inline void notify_mst(enum mst_interrupt_t evt);
inline void wait_for_int_host0(uint32_t evt);

#endif
