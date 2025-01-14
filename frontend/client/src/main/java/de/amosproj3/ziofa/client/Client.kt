// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow

data class Configuration(
    val vfsWrite: VfsWriteConfig?,
    val sysSendmsg: SysSendmsgConfig?,
    val uprobes: List<UprobeConfig>,
    val jniReferences: JniReferencesConfig?,
    val sysSigquit: SysSigquitConfig?,
)

data class VfsWriteConfig(val entries: Map<UInt, ULong>)

data class SysSendmsgConfig(val entries: Map<UInt, ULong>)

data class UprobeConfig(val fnName: String, val offset: ULong, var target: String, val pid: UInt?)

data class JniReferencesConfig(val pids: List<UInt>)

data class SysSigquitConfig(val pids: List<UInt>)

sealed class Event {
    data class VfsWrite(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: ULong,
        val fp: ULong,
        val bytesWritten: ULong,
    ) : Event()

    data class SysSendmsg(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: ULong,
        val fd: ULong,
        val durationNanoSecs: ULong,
    ) : Event()

    data class JniReferences(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: ULong,
        val jniMethodName: JniMethodName?,
    ) : Event() {
        enum class JniMethodName {
            AddLocalRef,
            DeleteLocalRef,
            AddGlobalRef,
            DeleteGlobalRef,
        }
    }

    data class SysSigquit(
        val pid: UInt,
        val tid: UInt,
        val timeStamp: ULong,
        val targetPid: ULong,
    ) : Event()
}

data class Process(val pid: UInt, val ppid: UInt, val state: String, val cmd: Command?)

data class StringResponse(val name: String)

data class Symbol(val method: String, val offset: ULong)

sealed class Command {
    data class Cmdline(val components: List<String>) : Command()

    data class Comm(val name: String) : Command()
}

interface Client {
    suspend fun serverCount(): Flow<UInt>

    suspend fun load()

    suspend fun attach(iface: String)

    suspend fun unload()

    suspend fun detach(iface: String)

    suspend fun startCollecting()

    suspend fun stopCollecting()

    suspend fun checkServer()

    suspend fun listProcesses(): List<Process>

    suspend fun getConfiguration(): Configuration

    suspend fun setConfiguration(configuration: Configuration)

    suspend fun getOdexFiles(pid: UInt): Flow<String>

    suspend fun getSoFiles(pid: UInt): Flow<String>

    suspend fun getSymbols(filePath: String): Flow<Symbol>

    suspend fun initStream(): Flow<Event>
}

interface ClientFactory {
    suspend fun connect(): Client
}
