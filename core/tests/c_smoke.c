/* Links the static library exactly as the Swift/C# shells will and drives
 * it through the C header. Build: see tests/c_smoke.sh */
#include "../include/odomouse_core.h"
#include <stdio.h>
#include <string.h>

static int fails = 0;
#define CHECK(c) do { if (!(c)) { printf("FAIL %s:%d %s\n", __FILE__, __LINE__, #c); fails++; } } while (0)

int main(int argc, char **argv) {
    const char *dir = argc > 1 ? argv[1] : "/tmp/odomouse-c-smoke";
    MkHandle *h = mk_open(dir, 18000);
    CHECK(h != NULL);
    mk_set_displays(h, "[{\"id\":1,\"internal\":true,\"scaleFactor\":2,"
                       "\"bounds\":{\"x\":0,\"y\":0,\"width\":1470,\"height\":956},\"widthMm\":294,\"heightMm\":191}]");
    mk_set_hooks_running(h, true);
    mk_mouse_move(h, 0, 0);
    mk_mouse_move(h, 300, 400);
    mk_mouse_down(h, 1);
    mk_wheel(h, 10, 10, 30);
    uint32_t e = mk_vc_from_mac(0x0E);
    CHECK(e == 18);
    CHECK(mk_vc_from_windows(0x4B, true) == 57419);
    CHECK(mk_vc_from_evdev(105) == 57419);
    CHECK(mk_vc_from_mac(0xFF) == 0);
    mk_key_down(h, e, 0);
    mk_key_up(h, e);
    mk_key_down(h, mk_vc_from_mac(0x08), MK_MOD_META);

    char *tick = mk_tick(h, 18000, true, 100, 100);
    printf("tick: %s\n", tick);
    CHECK(strstr(tick, "\"tray\":\"0.10") != NULL);
    mk_string_free(tick);

    char *live = mk_call(h, "getLive", NULL);
    CHECK(strstr(live, "\"keystrokes\":2") != NULL);
    CHECK(strstr(live, "\"hooksRunning\":true") != NULL);
    mk_string_free(live);

    char *w = mk_call(h, "getWidget", "[]");
    printf("widget: %.160s...\n", w);
    CHECK(strstr(w, "\"keys\":{\"count\":2") != NULL);
    mk_string_free(w);

    char *s = mk_call(h, "updateSettings", "[{\"trayShows\":\"keys\"}]");
    CHECK(strstr(s, "\"trayShows\":\"keys\"") != NULL);
    mk_string_free(s);
    char *tray = mk_tray_text(h);
    CHECK(strcmp(tray, "2") == 0);
    mk_string_free(tray);

    char *bad = mk_call(h, "nope", "not json");
    CHECK(strstr(bad, "error") != NULL);
    mk_string_free(bad);

    CHECK(mk_save(h));
    mk_close(h);
    mk_set_app(NULL, "x"); /* NULL handle is ignored */
    printf("core %s: %s\n", mk_version(), fails ? "FAILED" : "ok");
    return fails ? 1 : 0;
}
