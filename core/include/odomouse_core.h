/*
 * Odomouse core: C interface (Rust, see src/ffi.rs).
 *
 * - All strings are UTF-8. Every `char *` returned must be released with
 *   mk_string_free(). NULL is returned only for a NULL handle.
 * - The handle is thread-safe: input hooks may call from any thread.
 * - The core reads the clock itself; the shell passes the local UTC offset
 *   (seconds east of UTC, e.g. +18000 for Tashkent) to mk_open and mk_tick.
 */
#ifndef ODOMOUSE_CORE_H
#define ODOMOUSE_CORE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MkHandle MkHandle;

/* modifier bits for mk_key_down */
#define MK_MOD_CTRL  1u
#define MK_MOD_ALT   2u
#define MK_MOD_SHIFT 4u
#define MK_MOD_META  8u /* ⌘ on macOS, the Windows/Super key elsewhere */

/* Open or create <data_dir>/history.json and settings.json. */
MkHandle *mk_open(const char *data_dir, int32_t tz_offset_s);
/* Save and free. */
void mk_close(MkHandle *h);
void mk_string_free(char *s);
const char *mk_version(void);

/* ---- input events (cursor coordinates in the same space as display bounds) */
void mk_mouse_move(const MkHandle *h, double x, double y);
void mk_mouse_down(const MkHandle *h, uint32_t button); /* 1 left, 2 right, 3 middle */
void mk_wheel(const MkHandle *h, double x, double y, double points);
void mk_key_down(const MkHandle *h, uint32_t vc, uint32_t mods);
void mk_key_up(const MkHandle *h, uint32_t vc);

/* native key code -> core key id (0 = unknown key, skip it) */
uint32_t mk_vc_from_mac(uint16_t kvk);
uint32_t mk_vc_from_windows(uint32_t scan_code, bool extended);
uint32_t mk_vc_from_evdev(uint16_t code);

/* ---- state
 * displays: [{"id":"1","label":"Built-in","internal":true,"scaleFactor":2,
 *             "bounds":{"x":0,"y":0,"width":1470,"height":956},
 *             "widthMm":302,"heightMm":196,       physical size if known, else 0
 *             "estimatePpi":0}]                   units per inch guess, 0 = default
 */
void mk_set_displays(const MkHandle *h, const char *json);
void mk_set_hooks_running(const MkHandle *h, bool running);
void mk_set_app(const MkHandle *h, const char *name_or_null);

/* Once a second. Returns {"tray":"1.23 km","notify":null|{"title":..,"body":..}} */
char *mk_tick(const MkHandle *h, int32_t tz_offset_s, bool has_cursor, double x, double y);

/* UI bridge. method: getLive | getDashboard | getWrapped | getWidget |
 * updateSettings | exportCsv | resetData | clearApps | markDashboardSeen |
 * getReferences | getSettings | getDisplays. args_json: JSON array of arguments (or NULL). Returns JSON. */
char *mk_call(const MkHandle *h, const char *method, const char *args_json);

char *mk_tray_text(const MkHandle *h);
bool mk_save(const MkHandle *h);

#ifdef __cplusplus
}
#endif
#endif
