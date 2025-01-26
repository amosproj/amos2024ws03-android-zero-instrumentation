package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.flow

typealias Comm = Command.Comm
typealias Cmdline = Command.Cmdline

val processesList = listOf(
    Process(
        pid = 1u,
        ppid = 0u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/init", "second_stage"))
    ),
    Process(pid = 2u, ppid = 0u, state = "S", cmd = Comm(name = "kthreadd")),
    Process(pid = 3u, ppid = 2u, state = "I", cmd = Comm(name = "rcu_gp")),
    Process(pid = 4u, ppid = 2u, state = "I", cmd = Comm(name = "slub_flushwq")),
    Process(pid = 5u, ppid = 2u, state = "I", cmd = Comm(name = "netns")),
    Process(pid = 7u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/0:0H-kblockd")),
    Process(pid = 9u, ppid = 2u, state = "I", cmd = Comm(name = "mm_percpu_wq")),
    Process(pid = 11u, ppid = 2u, state = "I", cmd = Comm(name = "rcu_tasks_kthread")),
    Process(pid = 12u, ppid = 2u, state = "I", cmd = Comm(name = "rcu_tasks_trace_kthread")),
    Process(pid = 13u, ppid = 2u, state = "S", cmd = Comm(name = "ksoftirqd/0")),
    Process(pid = 14u, ppid = 2u, state = "I", cmd = Comm(name = "rcu_preempt")),
    Process(pid = 15u, ppid = 2u, state = "S", cmd = Comm(name = "rcub/0")),
    Process(pid = 16u, ppid = 2u, state = "S", cmd = Comm(name = "rcu_exp_gp_kthread_worker")),
    Process(pid = 17u, ppid = 2u, state = "S", cmd = Comm(name = "rcu_exp_par_gp_kthread_worker")),
    Process(pid = 18u, ppid = 2u, state = "S", cmd = Comm(name = "migration/0")),
    Process(pid = 19u, ppid = 2u, state = "S", cmd = Comm(name = "idle_inject/0")),
    Process(pid = 21u, ppid = 2u, state = "S", cmd = Comm(name = "cpuhp/0")),
    Process(pid = 22u, ppid = 2u, state = "S", cmd = Comm(name = "cpuhp/1")),
    Process(pid = 23u, ppid = 2u, state = "S", cmd = Comm(name = "idle_inject/1")),
    Process(pid = 24u, ppid = 2u, state = "S", cmd = Comm(name = "migration/1")),
    Process(pid = 25u, ppid = 2u, state = "S", cmd = Comm(name = "ksoftirqd/1")),
    Process(pid = 27u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/1:0H-kblockd")),
    Process(pid = 28u, ppid = 2u, state = "S", cmd = Comm(name = "cpuhp/2")),
    Process(pid = 29u, ppid = 2u, state = "S", cmd = Comm(name = "idle_inject/2")),
    Process(pid = 30u, ppid = 2u, state = "S", cmd = Comm(name = "migration/2")),
    Process(pid = 31u, ppid = 2u, state = "S", cmd = Comm(name = "ksoftirqd/2")),
    Process(pid = 34u, ppid = 2u, state = "S", cmd = Comm(name = "cpuhp/3")),
    Process(pid = 35u, ppid = 2u, state = "S", cmd = Comm(name = "idle_inject/3")),
    Process(pid = 36u, ppid = 2u, state = "S", cmd = Comm(name = "migration/3")),
    Process(pid = 37u, ppid = 2u, state = "S", cmd = Comm(name = "ksoftirqd/3")),
    Process(pid = 40u, ppid = 2u, state = "I", cmd = Comm(name = "inet_frag_wq")),
    Process(pid = 41u, ppid = 2u, state = "S", cmd = Comm(name = "kauditd")),
    Process(pid = 42u, ppid = 2u, state = "S", cmd = Comm(name = "khungtaskd")),
    Process(pid = 43u, ppid = 2u, state = "S", cmd = Comm(name = "oom_reaper")),
    Process(pid = 44u, ppid = 2u, state = "I", cmd = Comm(name = "writeback")),
    Process(pid = 45u, ppid = 2u, state = "S", cmd = Comm(name = "kcompactd0")),
    Process(pid = 46u, ppid = 2u, state = "S", cmd = Comm(name = "khugepaged")),
    Process(pid = 47u, ppid = 2u, state = "I", cmd = Comm(name = "cryptd")),
    Process(pid = 48u, ppid = 2u, state = "I", cmd = Comm(name = "kblockd")),
    Process(pid = 49u, ppid = 2u, state = "I", cmd = Comm(name = "blkcg_punt_bio")),
    Process(pid = 53u, ppid = 2u, state = "I", cmd = Comm(name = "edac-poller")),
    Process(pid = 54u, ppid = 2u, state = "I", cmd = Comm(name = "devfreq_wq")),
    Process(pid = 55u, ppid = 2u, state = "S", cmd = Comm(name = "watchdogd")),
    Process(pid = 56u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/3:1H-kblockd")),
    Process(pid = 58u, ppid = 2u, state = "S", cmd = Comm(name = "kswapd0")),
    Process(pid = 60u, ppid = 2u, state = "S", cmd = Comm(name = "erofs_worker/0")),
    Process(pid = 61u, ppid = 2u, state = "S", cmd = Comm(name = "erofs_worker/1")),
    Process(pid = 62u, ppid = 2u, state = "S", cmd = Comm(name = "erofs_worker/2")),
    Process(pid = 63u, ppid = 2u, state = "S", cmd = Comm(name = "erofs_worker/3")),
    Process(pid = 64u, ppid = 2u, state = "I", cmd = Comm(name = "kthrotld")),
    Process(pid = 65u, ppid = 2u, state = "S", cmd = Comm(name = "dmabuf-deferred-free-worker")),
    Process(pid = 66u, ppid = 2u, state = "I", cmd = Comm(name = "nvme-wq")),
    Process(pid = 67u, ppid = 2u, state = "I", cmd = Comm(name = "nvme-reset-wq")),
    Process(pid = 68u, ppid = 2u, state = "I", cmd = Comm(name = "nvme-delete-wq")),
    Process(pid = 69u, ppid = 2u, state = "I", cmd = Comm(name = "uas")),
    Process(pid = 70u, ppid = 2u, state = "I", cmd = Comm(name = "dm_bufio_cache")),
    Process(pid = 71u, ppid = 2u, state = "I", cmd = Comm(name = "mld")),
    Process(pid = 72u, ppid = 2u, state = "I", cmd = Comm(name = "ipv6_addrconf")),
    Process(pid = 74u, ppid = 2u, state = "S", cmd = Comm(name = "khvcd")),
    Process(pid = 76u, ppid = 2u, state = "S", cmd = Comm(name = "hwrng")),
    Process(pid = 81u, ppid = 2u, state = "S", cmd = Comm(name = "jbd2/vdd1-8")),
    Process(pid = 82u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 83u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:0")),
    Process(pid = 84u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:1")),
    Process(pid = 85u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:2")),
    Process(pid = 86u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:3")),
    Process(pid = 87u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:4")),
    Process(pid = 88u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:5")),
    Process(pid = 89u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 90u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 91u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 92u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 93u, ppid = 2u, state = "S", cmd = Comm(name = "f2fs_ckpt-254:5")),
    Process(pid = 94u, ppid = 2u, state = "S", cmd = Comm(name = "f2fs_flush-254:5")),
    Process(pid = 95u, ppid = 2u, state = "S", cmd = Comm(name = "f2fs_gc-254:5")),
    Process(
        pid = 98u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/init", "subcontext", "u:r:vendor_init:s0", "16"))
    ),
    Process(pid = 99u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/2:1H-kblockd")),
    Process(pid = 100u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/ueventd"))),
    Process(pid = 110u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:8")),
    Process(pid = 111u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:9")),
    Process(pid = 113u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:11")),
    Process(pid = 114u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:12")),
    Process(pid = 118u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:16")),
    Process(pid = 119u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:17")),
    Process(pid = 120u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:18")),
    Process(pid = 122u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:20")),
    Process(pid = 123u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:21")),
    Process(pid = 125u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:23")),
    Process(pid = 126u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:24")),
    Process(pid = 129u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:27")),
    Process(pid = 131u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:29")),
    Process(pid = 133u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:31")),
    Process(pid = 137u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:35")),
    Process(pid = 139u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:37")),
    Process(pid = 140u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:38")),
    Process(pid = 141u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:39")),
    Process(pid = 142u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:40")),
    Process(pid = 143u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:41")),
    Process(pid = 147u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 148u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 149u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 154u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/prng_seeder"))),
    Process(pid = 156u, ppid = 2u, state = "I", cmd = Comm(name = "cfg80211")),
    Process(pid = 157u, ppid = 2u, state = "S", cmd = Comm(name = "sugov:0")),
    Process(pid = 158u, ppid = 2u, state = "S", cmd = Comm(name = "sugov:1")),
    Process(pid = 159u, ppid = 2u, state = "S", cmd = Comm(name = "sugov:2")),
    Process(pid = 160u, ppid = 2u, state = "S", cmd = Comm(name = "sugov:3")),
    Process(pid = 161u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/logd"))),
    Process(pid = 162u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/lmkd"))),
    Process(
        pid = 163u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/servicemanager"))
    ),
    Process(
        pid = 164u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/system_ext/bin/hwservicemanager"))
    ),
    Process(pid = 168u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/sh"))),
    Process(pid = 169u, ppid = 2u, state = "S", cmd = Comm(name = "irq/18-goldfish_pipe_dprctd")),
    Process(pid = 170u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/vendor/bin/qemu-props"))),
    Process(pid = 172u, ppid = 2u, state = "S", cmd = Comm(name = "psimon")),
    Process(pid = 179u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/carwatchdogd"))),
    Process(
        pid = 180u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/cardisplayproxyd"))
    ),
    Process(
        pid = 181u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.automotive.evs-aidl-default-service"))
    ),
    Process(
        pid = 182u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/evsmanagerd", "--target", "hw/0"))
    ),
    Process(
        pid = 183u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(
            listOf(
                "/system/bin/vold",
                "--blkid_context=u:r:blkid:s0",
                "--blkid_untrusted_context=u:r:blkid_untrusted:s0",
                "--fsck_context=u:r:fsck:s0",
                "--fsck_untrusted_context=u:r:fsck_untrusted:s0"
            )
        )
    ),
    Process(pid = 190u, ppid = 2u, state = "I", cmd = Comm(name = "kdmflush/254:42")),
    Process(pid = 197u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/3:2-events")),
    Process(
        pid = 198u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/hw/android.system.suspend-service"))
    ),
    Process(
        pid = 199u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/carpowerpolicyd"))
    ),
    Process(
        pid = 200u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/keystore2", "/data/misc/keystore"))
    ),
    Process(
        pid = 201u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.atrace@1.0-service"))
    ),
    Process(
        pid = 202u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.security.keymint-service"))
    ),
    Process(
        pid = 204u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.automotive.vehicle@V3-emulator-service"))
    ),
    Process(pid = 211u, ppid = 2u, state = "I", cmd = Comm(name = "usbip_event")),
    Process(pid = 219u, ppid = 2u, state = "I", cmd = Comm(name = "blk_crypto_wq")),
    Process(pid = 240u, ppid = 2u, state = "S", cmd = Comm(name = "jbd2/dm-42-8")),
    Process(pid = 241u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 248u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/tombstoned"))),
    Process(pid = 293u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 294u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 295u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 296u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 297u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 298u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 299u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 300u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 301u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 302u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 303u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 304u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 305u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 306u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 307u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 308u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 309u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 310u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 312u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 313u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 314u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 315u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 316u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 317u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 318u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 320u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 321u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/0:2H-kblockd")),
    Process(pid = 323u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 325u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 326u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 327u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 328u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 329u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 330u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 331u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 332u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 333u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 334u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 335u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 336u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 337u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 338u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 339u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 340u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 341u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 342u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 343u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 344u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 345u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 346u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 347u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 348u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 349u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 350u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 351u, ppid = 2u, state = "I", cmd = Comm(name = "kverityd")),
    Process(pid = 352u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 353u, ppid = 2u, state = "I", cmd = Comm(name = "ext4-rsv-conver")),
    Process(pid = 368u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/3:2H-kblockd")),
    Process(
        pid = 382u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/dhcpclient", "-i", "wlan0", "--no-gateway"))
    ),
    Process(
        pid = 388u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.os.statsd/bin/statsd"))
    ),
    Process(pid = 389u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/netd"))),
    Process(pid = 390u, ppid = 1u, state = "S", cmd = Cmdline(listOf("zygote64"))),
    Process(
        pid = 397u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/system_ext/bin/hw/android.hidl.allocator@1.0-service"))
    ),
    Process(
        pid = 398u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.device.generic.car.emulator@1.0-protocanbus-service"))
    ),
    Process(
        pid = 399u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.audio.service-caremu"))
    ),
    Process(
        pid = 400u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.automotive.can@1.0-service"))
    ),
    Process(
        pid = 401u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.camera.provider.ranchu"))
    ),
    Process(
        pid = 402u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.camera.provider@2.7-service-google"))
    ),
    Process(
        pid = 403u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.gatekeeper@1.0-service.software"))
    ),
    Process(
        pid = 404u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.graphics.allocator@3.0-service.ranchu"))
    ),
    Process(
        pid = 405u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.health-service.automotive"))
    ),
    Process(
        pid = 406u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.media.c2@1.0-service-goldfish"))
    ),
    Process(
        pid = 407u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.sensors-service.multihal"))
    ),
    Process(
        pid = 408u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.thermal@2.0-service.mock"))
    ),
    Process(
        pid = 410u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.usb-service.example"))
    ),
    Process(
        pid = 411u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.wifi-service"))
    ),
    Process(
        pid = 416u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.bluetooth-service.default"))
    ),
    Process(
        pid = 420u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.broadcastradio-service.default"))
    ),
    Process(
        pid = 421u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.contexthub-service.example"))
    ),
    Process(
        pid = 422u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.graphics.composer3-service.ranchu"))
    ),
    Process(
        pid = 423u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.identity-service.example"))
    ),
    Process(
        pid = 424u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.automotive.ivn@V1-default-service"))
    ),
    Process(
        pid = 425u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.lights-service.example"))
    ),
    Process(
        pid = 426u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.power-service.example"))
    ),
    Process(
        pid = 427u,
        ppid = 389u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/iptables-restore", "--noflush", "-w", "-v"))
    ),
    Process(
        pid = 428u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.power.stats-service.example"))
    ),
    Process(
        pid = 429u,
        ppid = 389u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/ip6tables-restore", "--noflush", "-w", "-v"))
    ),
    Process(
        pid = 431u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.automotive.remoteaccess@V2-default-service"))
    ),
    Process(
        pid = 432u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.vibrator-service.example"))
    ),
    Process(
        pid = 442u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.authsecret/bin/hw/android.hardware.authsecret-service.example"))
    ),
    Process(
        pid = 445u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.cas/bin/hw/android.hardware.cas-service.example"))
    ),
    Process(
        pid = 449u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.neuralnetworks/bin/hw/android.hardware.neuralnetworks-service-sample-all"))
    ),
    Process(
        pid = 452u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.neuralnetworks/bin/hw/android.hardware.neuralnetworks-service-sample-limited"))
    ),
    Process(
        pid = 454u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.neuralnetworks/bin/hw/android.hardware.neuralnetworks-shim-service-sample"))
    ),
    Process(
        pid = 457u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.hardware.rebootescrow/bin/hw/android.hardware.rebootescrow-service.default"))
    ),
    Process(pid = 460u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/audioserver"))),
    Process(
        pid = 462u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/credstore", "/data/misc/credstore"))
    ),
    Process(pid = 463u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/gpuservice"))),
    Process(
        pid = 473u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/surfaceflinger"))
    ),
    Process(
        pid = 501u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.adbd/bin/adbd", "--root_seclabel=u:r:su:s0"))
    ),
    Process(pid = 509u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/traced_probes"))),
    Process(pid = 518u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/traced"))),
    Process(pid = 520u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/mdnsd"))),
    Process(
        pid = 535u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/logcat", "-f", "/dev/hvc1", "*:V"))
    ),
    Process(
        pid = 537u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(
            listOf(
                "/vendor/bin/bt_vhci_forwarder",
                "-virtio_console_dev=/dev/bluetooth0"
            )
        )
    ),
    Process(pid = 538u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/cameraserver"))),
    Process(pid = 540u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/incidentd"))),
    Process(pid = 542u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/installd"))),
    Process(
        pid = 543u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("media.extractor", "aextractor"))
    ),
    Process(
        pid = 544u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("media.metrics", "diametrics"))
    ),
    Process(pid = 545u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/mediaserver"))),
    Process(pid = 546u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/storaged"))),
    Process(pid = 547u, ppid = 1u, state = "S", cmd = Cmdline(listOf("/system/bin/wificond"))),
    Process(
        pid = 548u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/libgoldfish-rild"))
    ),
    Process(
        pid = 549u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("media.swcodec", "oid.media.swcodec/bin/mediaswcodec"))
    ),
    Process(
        pid = 550u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/gatekeeperd", "/data/misc/gatekeeper"))
    ),
    Process(
        pid = 591u,
        ppid = 501u,
        state = "S",
        cmd = Cmdline(listOf("/data/local/tmp/.studio/process-tracker", "--interval", "1000"))
    ),
    Process(pid = 685u, ppid = 390u, state = "S", cmd = Cmdline(listOf("system_server"))),
    Process(
        pid = 908u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(
            listOf(
                "/vendor/bin/hw/wpa_supplicant",
                "-Dnl80211",
                "-iwlan0",
                "-c/vendor/etc/wifi/wpa_supplicant.conf",
                "-g@android:wpa_wlan0"
            )
        )
    ),
    Process(pid = 939u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car"))),
    Process(
        pid = 955u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/vendor/bin/hw/android.hardware.gnss-service.ranchu"))
    ),
    Process(pid = 976u, ppid = 390u, state = "S", cmd = Cmdline(listOf("webview_zygote"))),
    Process(pid = 1054u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car"))),
    Process(
        pid = 1065u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.experimentalcar"))
    ),
    Process(
        pid = 1066u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/system/bin/com.android.car.procfsinspector"))
    ),
    Process(
        pid = 1101u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.networkstack.process"))
    ),
    Process(pid = 1103u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.se"))),
    Process(pid = 1108u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.phone"))),
    Process(pid = 1121u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.systemui"))),
    Process(pid = 1139u, ppid = 390u, state = "S", cmd = Cmdline(listOf("android.ext.services"))),
    Process(
        pid = 1181u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.permissioncontroller"))
    ),
    Process(pid = 1232u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.bluetooth"))),
    Process(pid = 1284u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car.rotary"))),
    Process(
        pid = 1413u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.car.carlauncher"))
    ),
    Process(
        pid = 1494u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.inputmethod.latin"))
    ),
    Process(pid = 1496u, ppid = 390u, state = "S", cmd = Cmdline(listOf("android.ext.services"))),
    Process(
        pid = 1549u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.providers.media.module"))
    ),
    Process(pid = 1627u, ppid = 390u, state = "S", cmd = Cmdline(listOf("android.car.cluster"))),
    Process(pid = 1696u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car.radio"))),
    Process(pid = 1715u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car.media"))),
    Process(
        pid = 1764u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.providers.media.module"))
    ),
    Process(
        pid = 1994u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.providers.calendar"))
    ),
    Process(pid = 2109u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.car.dialer"))),
    Process(
        pid = 2128u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.car.messenger"))
    ),
    Process(
        pid = 2249u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.managedprovisioning"))
    ),
    Process(
        pid = 2287u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.packageinstaller"))
    ),
    Process(
        pid = 2307u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.providers.calendar"))
    ),
    Process(pid = 2327u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.shell"))),
    Process(
        pid = 2345u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.statementservice"))
    ),
    Process(
        pid = 2363u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.google.android.car.networking.preferenceupdater"))
    ),
    Process(
        pid = 2382u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.externalstorage"))
    ),
    Process(
        pid = 2399u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.externalstorage"))
    ),
    Process(pid = 2416u, ppid = 390u, state = "S", cmd = Cmdline(listOf("android.process.media"))),
    Process(pid = 3295u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.keychain"))),
    Process(pid = 3311u, ppid = 390u, state = "S", cmd = Cmdline(listOf("com.android.keychain"))),
    Process(
        pid = 3327u,
        ppid = 390u,
        state = "S",
        cmd = Cmdline(listOf("com.android.permissioncontroller"))
    ),
    Process(pid = 3460u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/2:0-virtio_vsock")),
    Process(pid = 6889u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/2:2H")),
    Process(pid = 8389u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u8:0-events_unbound")),
    Process(pid = 8668u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/0:0-events")),
    Process(pid = 9180u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/2:2-virtio_vsock")),
    Process(pid = 9533u, ppid = 501u, state = "S", cmd = Cmdline(listOf("-/system/bin/sh"))),
    Process(
        pid = 9535u,
        ppid = 9533u,
        state = "S",
        cmd = Cmdline(
            listOf(
                "/data/local/tmp/runner",
                "{\"command\":\"/data/local/tmp/backend-daemon\",\"args\":[],\"env\":{\"RUST_LOG\":\"error\"},\"root\":true}"
            )
        )
    ),
    Process(
        pid = 9536u,
        ppid = 9535u,
        state = "S",
        cmd = Cmdline(
            listOf(
                "/data/local/tmp/runner",
                "{\"command\":\"/data/local/tmp/backend-daemon\",\"args\":[],\"env\":{\"RUST_LOG\":\"error\"},\"root\":false}"
            )
        )
    ),
    Process(
        pid = 9537u,
        ppid = 9536u,
        state = "S",
        cmd = Cmdline(listOf("/data/local/tmp/backend-daemon"))
    ),
    Process(pid = 9800u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/1:2H")),
    Process(pid = 11692u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u9:0-blk_crypto_wq")),
    Process(pid = 12935u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/3:0-mm_percpu_wq")),
    Process(pid = 13037u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/0:2")),
    Process(pid = 13649u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/1:1-virtio_vsock")),
    Process(pid = 14003u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/1:2-virtio_vsock")),
    Process(pid = 14014u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u9:1-blk_crypto_wq")),
    Process(pid = 14028u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u8:2-events_unbound")),
    Process(pid = 14517u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u9:2-blk_crypto_wq")),
    Process(
        pid = 14518u,
        ppid = 2u,
        state = "I",
        cmd = Comm(name = "kworker/u9:3-fscrypt_read_queue")
    ),
    Process(
        pid = 14529u,
        ppid = 2u,
        state = "I",
        cmd = Comm(name = "kworker/u9:4-fscrypt_read_queue")
    ),
    Process(pid = 14547u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u8:1-events_unbound")),
    Process(pid = 14617u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/1:0-virtio_vsock")),
    Process(
        pid = 14626u,
        ppid = 1u,
        state = "S",
        cmd = Cmdline(listOf("/apex/com.android.art/bin/artd"))
    ),
    Process(pid = 14648u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u8:3-events_unbound")),
    Process(pid = 14660u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/u8:4-events_unbound")),
    Process(
        pid = 14667u,
        ppid = 501u,
        state = "S",
        cmd = Cmdline(listOf("logcat", "-v", "long", "-v", "epoch"))
    ),
    Process(pid = 14679u, ppid = 390u, state = "S", cmd = Cmdline(listOf("de.amosproj3.ziofa"))),
    Process(pid = 14714u, ppid = 2u, state = "I", cmd = Comm(name = "kworker/0:1-events")),
)

val mockOdexFileFlow = flow {
    emit("/system/framework/oat/x86_64/android.test.base.odex")
    emit("/system/framework/oat/x86_64/android.hidl.base-V1.0-java.odex")
    emit("/system/framework/oat/x86_64/org.apache.http.legacy.odex")
    emit("/system/framework/oat/x86_64/android.hidl.manager-V1.0-java.odex")
    emit("/system_ext/framework/oat/x86_64/androidx.window.sidecar.odex")
    emit(
        "/data/app/~~0cD8TtY5ggbzXOrlKANgwQ==/de.amosproj3.ziofa-Sm8ZemAtgxCr5VAK1Cwi8Q==/oat/x86_64/base.odex"
    )

    emit("/system_ext/framework/oat/x86_64/androidx.window.extensions.odex")
}

val mockSoFileFlow = flow {
    emit("/system/lib64/liblog.so")
    emit("/vendor/lib64/libdrm.so")
    emit("/vendor/lib64/android.hardware.graphics.mapper@3.0.so")
    emit("/system/lib64/android.hardware.power-V5-ndk.so")
    emit("/system/lib64/android.hardware.graphics.mapper@2.0.so")
    emit("/system/lib64/android.hardware.media.c2@1.2.so")

    emit("/system/lib64/android.hardware.renderscript@1.0.so")
}

val mockSymbolFlow =
    flow {
        emit(
            Symbol(
                method =
                "void androidx.compose.material3.SearchBarDefaults\$InputField\$1\$1.<init>(kotlin.jvm.functions.Function1)",
                offset = 6012800u,
            )
        )
        emit(
            Symbol(
                method =
                "void kotlin.collections.ArraysKt___ArraysKt\$asSequence\$\$inlined\$Sequence\$2.<init>(byte[])",
                offset = 5915712u,
            )
        )
        emit(
            Symbol(
                method =
                "boolean androidx.compose.ui.platform.ViewLayer\$Companion.getHasRetrievedMethod()",
                offset = 24010112u,
            )
        )
        emit(
            Symbol(
                method =
                "androidx.core.app.NotificationCompat\$BubbleMetadata androidx.core.app.NotificationCompat\$BubbleMetadata\$Api29Impl.fromPlatform(android.app.Notification\$BubbleMetadata)",
                offset = 25453376u,
            )
        )
        emit(
            Symbol(
                method = "byte androidx.emoji2.text.flatbuffer.FlexBuffers\$Blob.get(int)",
                offset = 26906336u,
            )
        )
    }
