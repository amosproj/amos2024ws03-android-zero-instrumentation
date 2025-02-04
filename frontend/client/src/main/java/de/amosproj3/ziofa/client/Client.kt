// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import kotlinx.datetime.Instant
import kotlin.time.Duration

data class Configuration(
    val vfsWrite: VfsWriteConfig?,
    val sysSendmsg: SysSendmsgConfig?,
    val uprobes: List<UprobeConfig>,
    val jniReferences: JniReferencesConfig?,
    val sysSigquit: SysSigquitConfig?,
    val gc: GcConfig?,
    val sysFdTracking: SysFdTrackingConfig?,
)

data class VfsWriteConfig(val pids: List<UInt>)

data class SysSendmsgConfig(val entries: Map<UInt, ULong>)

data class UprobeConfig(val fnName: String, val offset: ULong, var target: String, val pid: UInt?)

data class JniReferencesConfig(val pids: List<UInt>)

data class SysSigquitConfig(val pids: List<UInt>)

/** Warning: This is a workaround. The configuration is not persisted! */
data class GcConfig(val pids: Set<UInt>)

data class SysFdTrackingConfig(val pids: List<UInt>)

sealed class Event {
    data class VfsWrite(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: Instant,
        val fp: String,
        val bytesWritten: ULong,
    ) : Event()

    data class SysSendmsg(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: Instant,
        val duration: Duration,
    ) : Event()

    data class JniReferences(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: Instant,
        val jniMethodName: JniMethodName?,
    ) : Event() {
        enum class JniMethodName {
            AddLocalRef,
            DeleteLocalRef,
            AddGlobalRef,
            DeleteGlobalRef,
            Undefined
        }
    }

    data class SysSigquit(
        val pid: UInt,
        val tid: UInt,
        val timeStamp: Instant,
        val targetPid: ULong,
    ) : Event()

    data class Gc(
        val pid: UInt,
        val tid: UInt,
        var targetFootprint: ULong,
        var numBytesAllocated: ULong,
        var gcsCompleted: UInt,
        var gcCause: UInt,
        var durationNs: ULong,
        var freedObjects: ULong,
        var freedBytes: Long,
        var freedLosObjects: ULong,
        var freedLosBytes: Long,
        var pauseTimes: List<ULong>,
    ) : Event()

    data class SysFdTracking(
        val pid: UInt,
        val tid: UInt,
        val timeStamp: Instant,
        val fdAction: SysFdAction?,
    ) : Event() {
        enum class SysFdAction {
            Created,
            Destroyed,
            Undefined
        }
    }
}

data class Process(val pid: UInt, val ppid: UInt, val state: String, val cmd: Command?)

data class StringResponse(val name: String)

data class Symbol(val method: String, val offset: ULong)

sealed class Command {
    data class Cmdline(val components: List<String>) : Command()

    data class Comm(val name: String) : Command()
}

interface Client {
    suspend fun listProcesses(): List<Process>

    suspend fun getConfiguration(): Configuration

    suspend fun setConfiguration(configuration: Configuration)

    suspend fun initStream(): Flow<Event>

    suspend fun indexSymbols()
}

interface ClientFactory {
    suspend fun connect(): Client
}
