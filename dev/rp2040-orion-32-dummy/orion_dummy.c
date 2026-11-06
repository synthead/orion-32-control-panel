// orion_dummy.c -- RP2040 dummy "Antelope Audio Orion 32" (23e5:a040)
// Composite descriptor copied from a real unit's lsusb dump, so Windows binds
// its orion32 driver the same way. Audio interfaces are descriptors only: the
// device answers the control requests but streams no audio.
//
// Needs next to this file: tusb_config.h and CMakeLists.txt (unchanged).

#include "pico/stdlib.h"
#include "bsp/board_api.h"
#include "tusb.h"
#include "device/usbd_pvt.h"
#include <string.h>

// ---------------------------------------------------------------- descriptors

#define EPNUM_HID_OUT  0x01
#define EPNUM_HID_IN   0x82
#define EPNUM_AUDIO_OUT 0x05
#define EPNUM_AUDIO_IN  0x84

// The real unit is high speed (3 x 864 bytes per microframe); the RP2040 is
// full speed, so claim the 1023 byte maximum instead. The dump says 32 channels
// of 24 bit, which the dummy keeps.
#define AUDIO_EP_SIZE  1023

// Everything between (not including) the IAD and the HID interface.
#define AUDIO_FUNC_LEN 194
#define AC_TOTAL_LEN   75

#define ITF_NUM_HID    3

tusb_desc_device_t const desc_device = {
    .bLength            = sizeof(tusb_desc_device_t),
    .bDescriptorType    = TUSB_DESC_DEVICE,
    .bcdUSB             = 0x0200,
    .bDeviceClass       = TUSB_CLASS_MISC,
    .bDeviceSubClass    = MISC_SUBCLASS_COMMON,
    .bDeviceProtocol    = MISC_PROTOCOL_IAD,
    .bMaxPacketSize0    = CFG_TUD_ENDPOINT0_SIZE,
    .idVendor           = 0x23E5,
    .idProduct          = 0xA040,
    .bcdDevice          = 0x0100,
    .iManufacturer      = 1,
    .iProduct           = 2,
    .iSerialNumber      = 0,
    .bNumConfigurations = 1
};
uint8_t const *tud_descriptor_device_cb(void) { return (uint8_t const *)&desc_device; }

uint8_t const desc_hid_report[] = {
    0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01,
    0x09, 0x02, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x40, 0x81, 0x02,
    0x09, 0x03, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x40, 0x91, 0x02,
    0xC0
};
uint8_t const *tud_hid_descriptor_report_cb(uint8_t itf) { return desc_hid_report; }

#define CONFIG_TOTAL_LEN (TUD_CONFIG_DESC_LEN + 8 + AUDIO_FUNC_LEN + TUD_HID_INOUT_DESC_LEN)

#define U16_LE(x) ((x) & 0xFF), (((x) >> 8) & 0xFF)

uint8_t const desc_configuration[] = {
    TUD_CONFIG_DESCRIPTOR(1, 4, 0, CONFIG_TOTAL_LEN, TUSB_DESC_CONFIG_ATT_SELF_POWERED, 100),

    // Interface association: audio function, interfaces 0..2, UAC2
    8, TUSB_DESC_INTERFACE_ASSOCIATION, 0, 3, TUSB_CLASS_AUDIO, 0x00, 0x20, 0,

    // IF0: AudioControl
    9, TUSB_DESC_INTERFACE, 0, 0, 0, TUSB_CLASS_AUDIO, 0x01, 0x20, 0,
    // header: bcdADC 2.00, category 10, total length, no controls
    9, 0x24, 0x01, U16_LE(0x0200), 0x0A, U16_LE(AC_TOTAL_LEN), 0x00,
    // clock source 1: internal variable, freq control read/write
    8, 0x24, 0x0A, 0x01, 0x02, 0x03, 0x00, 0x00,
    // input terminal 7: USB streaming, 1 ch, clock 1, assoc 8, string 4
    17, 0x24, 0x02, 0x07, U16_LE(0x0101), 0x08, 0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0x04,
    // output terminal 8: digital audio interface, source 7, clock 1, string 4
    12, 0x24, 0x03, 0x08, U16_LE(0x0602), 0x00, 0x07, 0x01, 0, 0, 0x04,
    // input terminal 9: mic array, 1 ch, clock 1, assoc 10, string 5
    17, 0x24, 0x02, 0x09, U16_LE(0x0206), 0x0A, 0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0x05,
    // output terminal 10: USB streaming, source 9, clock 1, string 5
    12, 0x24, 0x03, 0x0A, U16_LE(0x0101), 0x00, 0x09, 0x01, 0, 0, 0x05,

    // IF1: playback stream, alt 0 (idle) and alt 1 (EP5 OUT)
    9, TUSB_DESC_INTERFACE, 1, 0, 0, TUSB_CLASS_AUDIO, 0x02, 0x20, 4,
    9, TUSB_DESC_INTERFACE, 1, 1, 1, TUSB_CLASS_AUDIO, 0x02, 0x20, 4,
    // AS general: terminal 7, format type I, PCM, 32 channels
    16, 0x24, 0x01, 0x07, 0x00, 0x01, 0x01, 0, 0, 0, 32, 0, 0, 0, 0, 0,
    // format type I: 3 byte slots, 24 bit
    6, 0x24, 0x02, 0x01, 0x03, 0x18,
    7, TUSB_DESC_ENDPOINT, EPNUM_AUDIO_OUT, 0x05, U16_LE(AUDIO_EP_SIZE), 1,
    8, 0x25, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,

    // IF2: recording stream, alt 0 (idle) and alt 1 (EP4 IN, implicit feedback)
    9, TUSB_DESC_INTERFACE, 2, 0, 0, TUSB_CLASS_AUDIO, 0x02, 0x20, 5,
    9, TUSB_DESC_INTERFACE, 2, 1, 1, TUSB_CLASS_AUDIO, 0x02, 0x20, 5,
    16, 0x24, 0x01, 0x0A, 0x00, 0x01, 0x01, 0, 0, 0, 32, 0, 0, 0, 0, 0,
    6, 0x24, 0x02, 0x01, 0x03, 0x18,
    7, TUSB_DESC_ENDPOINT, EPNUM_AUDIO_IN, 0x25, U16_LE(AUDIO_EP_SIZE), 1,
    8, 0x25, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,

    // IF3: HID, EP2 IN + EP1 OUT, 64 bytes, interval 7 (same order as the real unit)
    9, TUSB_DESC_INTERFACE, ITF_NUM_HID, 0, 2, TUSB_CLASS_HID, 0, 0, 0,
    9, HID_DESC_TYPE_HID, U16_LE(0x0111), 0, 1, HID_DESC_TYPE_REPORT, U16_LE(sizeof(desc_hid_report)),
    7, TUSB_DESC_ENDPOINT, EPNUM_HID_IN,  TUSB_XFER_INTERRUPT, U16_LE(64), 7,
    7, TUSB_DESC_ENDPOINT, EPNUM_HID_OUT, TUSB_XFER_INTERRUPT, U16_LE(64), 7
};

_Static_assert(sizeof(desc_configuration) == CONFIG_TOTAL_LEN,
               "config descriptor length does not match CONFIG_TOTAL_LEN");

uint8_t const *tud_descriptor_configuration_cb(uint8_t index) { return desc_configuration; }

static char const *string_desc_arr[] = {
    (const char[]){0x09, 0x04},   // 0: English
    "Antelope Audio",             // 1
    "Orion 32",                   // 2
    "",                           // 3: unknown
    "Orion 32 Playback",          // 4
    "Orion 32 Recording",         // 5
};
static uint16_t desc_str[32];
uint16_t const *tud_descriptor_string_cb(uint8_t index, uint16_t langid) {
    uint8_t n;
    if (index == 0) { desc_str[1] = 0x0409; n = 1; }
    else {
        if (index >= sizeof(string_desc_arr) / sizeof(string_desc_arr[0])) return NULL;
        char const *s = string_desc_arr[index];
        n = strlen(s); if (n > 31) n = 31;
        for (uint8_t i = 0; i < n; i++) desc_str[1 + i] = s[i];
    }
    desc_str[0] = (TUSB_DESC_STRING << 8) | (2 * n + 2);
    return desc_str;
}

// ------------------------------------------------- audio function (stub driver)
//
// TinyUSB has no driver for these interfaces with CFG_TUD_AUDIO = 0 and would
// refuse to configure the device, so this app driver claims the whole audio
// function, tracks alt settings and answers the UAC2 clock requests.

static uint8_t  audio_alt[3];
static uint32_t audio_freq = 48000;

typedef struct __attribute__((packed)) {
    uint16_t n;
    uint32_t min, max, res;
} freq_range_t;

static freq_range_t const audio_freq_range = { 1, 44100, 192000, 0 };

static void audio_init(void) {}
static bool audio_deinit(void) { return true; }
static void audio_reset(uint8_t rhport) { (void)rhport; memset(audio_alt, 0, sizeof(audio_alt)); }

static uint16_t audio_open(uint8_t rhport, tusb_desc_interface_t const *itf, uint16_t max_len) {
    (void)rhport;
    if (itf->bInterfaceClass != TUSB_CLASS_AUDIO) return 0;
    if (max_len < AUDIO_FUNC_LEN) return 0;

    return AUDIO_FUNC_LEN;
}

static bool audio_control_xfer(uint8_t rhport, uint8_t stage, tusb_control_request_t const *r) {
    if (stage != CONTROL_STAGE_SETUP) return true;

    if (r->bmRequestType_bit.type == TUSB_REQ_TYPE_STANDARD) {
        uint8_t itf = tu_u16_low(r->wIndex);
        if (itf >= sizeof(audio_alt)) return false;

        if (r->bRequest == TUSB_REQ_SET_INTERFACE) {
            audio_alt[itf] = (uint8_t)tu_u16_low(r->wValue);
            return tud_control_status(rhport, r);
        }
        if (r->bRequest == TUSB_REQ_GET_INTERFACE) {
            return tud_control_xfer(rhport, r, &audio_alt[itf], 1);
        }

        return false;
    }

    if (r->bmRequestType_bit.type != TUSB_REQ_TYPE_CLASS) return false;

    uint8_t entity = tu_u16_high(r->wIndex);
    uint8_t cs     = tu_u16_high(r->wValue);
    bool    in     = r->bmRequestType_bit.direction == TUSB_DIR_IN;

    if (entity != 1) return false;                             // only the clock source

    if (cs == 0x01) {                                          // sampling frequency
        if (r->bRequest == 0x01) {                             // CUR
            return tud_control_xfer(rhport, r, &audio_freq, sizeof(audio_freq));
        }
        if (r->bRequest == 0x02 && in) {                       // RANGE
            return tud_control_xfer(rhport, r, (void *)(uintptr_t)&audio_freq_range,
                                    sizeof(audio_freq_range));
        }
    } else if (cs == 0x02 && r->bRequest == 0x01 && in) {     // clock valid
        static uint8_t valid = 1;
        return tud_control_xfer(rhport, r, &valid, 1);
    }

    return false;
}

static bool audio_xfer(uint8_t rhport, uint8_t ep_addr, xfer_result_t result, uint32_t n) {
    (void)rhport; (void)ep_addr; (void)result; (void)n;
    return true;
}

static usbd_class_driver_t const audio_driver = {
#if CFG_TUSB_DEBUG >= CFG_TUSB_DEBUG_PRINTF
    .name             = "ORION_AUDIO",
#endif
    .init             = audio_init,
    .deinit           = audio_deinit,
    .reset            = audio_reset,
    .open             = audio_open,
    .control_xfer_cb  = audio_control_xfer,
    .xfer_cb          = audio_xfer,
    .sof              = NULL,
};

usbd_class_driver_t const *usbd_app_driver_get_cb(uint8_t *driver_count) {
    *driver_count = 1;

    return &audio_driver;
}

// ------------------------------------------------------------------ HID logic

#define Q_LEN 32
static uint8_t q[Q_LEN][64];
static uint8_t q_head, q_tail;

static void q_push(const uint8_t *p) {
    uint8_t next = (q_head + 1) % Q_LEN;
    if (next == q_tail) return;
    memcpy(q[q_head], p, 64);
    q_head = next;
}

// VU packet doubles as device state
static uint8_t vu[64] = { 0x73 };

static void reply(uint8_t what) {
    uint8_t p[64] = {0};
    p[0] = 0x75; p[4] = 0x10; p[12] = what;
    switch (what) {
    case 0: p[16] = 1; p[17] = 145; break;                  // firmware 1.145
    case 1: memcpy(&p[16], "Orion", 5);                     // device name (offset inferred)
            memcpy(&p[36], "9000000000001", 13);            // serial, 13 chars
            memcpy(&p[52], "4.2", 4); break;                // hardware version
    case 2:                                                  // routes, garbage is fine
        for (int i = 0; i < 7; i++) { memset(p + 16, 0, 48); p[16] = i; q_push(p); }
        return;
    case 3: p[16] = 1; break;                               // registration: registered = 1
    case 5: memset(&p[16], 0x20, 33); break;                // pans default
    default: break;                                          // 4, 6: zeros
    }
    q_push(p);
}

void tud_hid_set_report_cb(uint8_t itf, uint8_t id, hid_report_type_t t,
                           uint8_t const *b, uint16_t len) {
    if (len < 64) return;
    if (b[0] == 0x74) { reply(b[12]); return; }
    if (b[0] != 0x70) return;
    const uint8_t *pl = b + 16;
    if (b[4] == 0x12) {
        if      (pl[0] == 0x03) vu[19] = pl[1];  // clock source
        else if (pl[0] == 0x0a) vu[20] = pl[1];  // oscillators
        else if (pl[0] == 0x0e) vu[25] = pl[1];  // usb channels
        else if (pl[0] == 0x08) vu[28] = pl[1];  // line out
    } else if (b[4] == 0x16 && pl[0] == 0xc2) vu[18] = pl[2];  // sample rate
    else if (b[4] == 0x13 && pl[0] == 0x44) vu[26] = pl[1];    // brightness
}

uint16_t tud_hid_get_report_cb(uint8_t i, uint8_t id, hid_report_type_t t,
                               uint8_t *b, uint16_t l) { return 0; }

// ------------------------------------------------------------------------ main

int main(void) {
    board_init();
    tusb_init();

    // Powered-on state, copied from a capture of a real unit.
    vu[8]  = 0x40;
    vu[16] = 0x08;
    vu[17] = 0x01 | 0x02 | 0x08;                   // 0x01 = on (per capture), LOCK, USB (drop 0x08 for non-USB)
    vu[18] = 0x01;                                 // sample rate
    vu[20] = 0xC0;
    vu[21] = 0x00; vu[22] = 0xAC; vu[23] = 0x44;   // 44100 Hz
    vu[25] = 0x03;
    vu[26] = 0xFF;                                 // brightness
    vu[27] = 0x10;
    vu[29] = 0x40;
    vu[30] = 0x71;                                 // VU sources

    uint32_t last = 0;
    while (1) {
        tud_task();

        if (tud_hid_ready() && q_tail != q_head) {
            tud_hid_report(0, q[q_tail], 64);
            q_tail = (q_tail + 1) % Q_LEN;
        }

        uint32_t now = to_ms_since_boot(get_absolute_time());
        if (now - last >= 10) {
            last = now;
            vu[31] ^= 1;
            for (int i = 32; i < 64; i++) vu[i] = (now / 20 + i) & 0x3f;
            q_push(vu);
        }
    }
}
