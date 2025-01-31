// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.client.jniMethodNameFromI32
import uniffi.client.sysFdActionFromI32
import uniffi.shared.Cmd
import uniffi.shared.EventData
import uniffi.shared.EventType
import uniffi.shared.JniMethodName
import uniffi.shared.SysFdAction

var gcPids = setOf<UInt>()

private fun uniffi.shared.Process.into() =
    Process(
        pid,
        ppid,
        state,
        cmd =
            when (val c = cmd) {
                is Cmd.Comm -> Command.Comm(c.v1)
                is Cmd.Cmdline -> Command.Cmdline(c.v1.args)
                null -> null
            },
    )

private fun uniffi.shared.Event.into() =
    when (val x = eventType) {
        is EventType.Log ->
            when (val d = x.v1.eventData) {
                is EventData.VfsWrite ->
                    Event.VfsWrite(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        beginTimeStamp = d.v1.beginTimeStamp,
                        fp = d.v1.fp,
                        bytesWritten = d.v1.bytesWritten,
                    )

                is EventData.SysSendmsg ->
                    Event.SysSendmsg(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        beginTimeStamp = d.v1.beginTimeStamp,
                        fd = d.v1.fd,
                        durationNanoSecs = d.v1.durationNanoSec,
                    )

                is EventData.JniReferences ->
                    Event.JniReferences(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        beginTimeStamp = d.v1.beginTimeStamp,
                        jniMethodName =
                            when (jniMethodNameFromI32(d.v1.jniMethodName)) {
                                JniMethodName.ADD_LOCAL_REF ->
                                    Event.JniReferences.JniMethodName.AddLocalRef

                                JniMethodName.DELETE_LOCAL_REF ->
                                    Event.JniReferences.JniMethodName.DeleteLocalRef

                                JniMethodName.ADD_GLOBAL_REF ->
                                    Event.JniReferences.JniMethodName.AddGlobalRef

                                JniMethodName.DELETE_GLOBAL_REF ->
                                    Event.JniReferences.JniMethodName.DeleteGlobalRef

                                JniMethodName.UNDEFINED -> null
                            },
                    )

                is EventData.SysSigquit ->
                    Event.SysSigquit(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        timeStamp = d.v1.timeStamp,
                        targetPid = d.v1.targetPid,
                    )

                is EventData.Gc ->
                    Event.Gc(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        targetFootprint = d.v1.targetFootprint,
                        numBytesAllocated = d.v1.numBytesAllocated,
                        gcsCompleted = d.v1.gcsCompleted,
                        gcCause = d.v1.gcCause,
                        durationNs = d.v1.durationNs,
                        freedObjects = d.v1.freedObjects,
                        freedBytes = d.v1.freedBytes,
                        freedLosObjects = d.v1.freedLosObjects,
                        freedLosBytes = d.v1.freedLosBytes,
                        pauseTimes = d.v1.pauseTimes,
                    )

                is EventData.SysFdTracking ->
                    Event.SysFdTracking(
                        pid = d.v1.pid,
                        tid = d.v1.tid,
                        timeStamp = d.v1.timeStamp,
                        fdAction =
                            when (sysFdActionFromI32(d.v1.fdAction)) {
                                SysFdAction.CREATED -> Event.SysFdTracking.SysFdAction.Created
                                SysFdAction.DESTROYED -> Event.SysFdTracking.SysFdAction.Destroyed
                                SysFdAction.UNDEFINED -> null
                            },
                    )

                null -> null
            }

        is EventType.TimeSeries -> null
        null -> null
    }

private fun uniffi.shared.Configuration.into() =
    Configuration(
        vfsWrite = vfsWrite?.let { VfsWriteConfig(entries = it.entries) },
        sysSendmsg = sysSendmsg?.let { SysSendmsgConfig(entries = it.entries) },
        uprobes =
            uprobes.map {
                UprobeConfig(
                    fnName = it.fnName,
                    offset = it.offset,
                    target = it.target,
                    pid = it.pid,
                )
            },
        jniReferences = jniReferences?.let { JniReferencesConfig(it.pids) },
        sysSigquit = sysSigquit?.let { SysSigquitConfig(pids = it.pids) },
        gc = gc?.let { GcConfig(gcPids) },
        sysFdTracking = sysFdTracking?.let { SysFdTrackingConfig(it.pids) },
    )

private fun Configuration.into() =
    uniffi.shared.Configuration(
        vfsWrite = vfsWrite?.let { uniffi.shared.VfsWriteConfig(it.entries) },
        sysSendmsg = sysSendmsg?.let { uniffi.shared.SysSendmsgConfig(it.entries) },
        uprobes =
            uprobes.map {
                uniffi.shared.UprobeConfig(
                    fnName = it.fnName,
                    offset = it.offset,
                    target = it.target,
                    pid = it.pid,
                )
            },
        jniReferences = jniReferences?.let { uniffi.shared.JniReferencesConfig(it.pids) },
        sysSigquit = sysSigquit?.let { uniffi.shared.SysSigquitConfig(it.pids) },
        gc =
            gc?.let {
                gcPids = it.pids
                uniffi.shared.GcConfig()
            },
        sysFdTracking = sysFdTracking?.let { uniffi.shared.SysFdTrackingConfig(it.pids) },
    )

private fun uniffi.shared.StringResponse.into() = StringResponse(name)

private fun uniffi.shared.Symbol.into() = Symbol(method, offset)

class RustClient(private val inner: uniffi.client.Client) : Client {

    override suspend fun serverCount(): Flow<UInt> = inner.serverCountFlow()

    override suspend fun load() = inner.load()

    override suspend fun attach(iface: String) = inner.attach(iface)

    override suspend fun unload() = inner.unload()

    override suspend fun detach(iface: String) = inner.detach(iface)

    override suspend fun startCollecting() = inner.startCollecting()

    override suspend fun stopCollecting() = inner.stopCollecting()

    override suspend fun listProcesses(): List<Process> = inner.listProcesses().map { it.into() }

    override suspend fun getConfiguration(): Configuration = inner.getConfiguration().into()

    // TODO remove the workarounds
    override suspend fun setConfiguration(configuration: Configuration) =
        inner.setConfiguration(configuration.into())

    override suspend fun getOdexFiles(pid: UInt): Flow<String> =
        inner.getOdexFilesFlow(pid).mapNotNull { it.into().name }

    override suspend fun getSoFiles(pid: UInt): Flow<String> =
        inner.getSoFilesFlow(pid).mapNotNull { it.into().name }

    override suspend fun getSymbols(filePath: String): Flow<Symbol> =
        inner.getSymbolFlow(filePath).mapNotNull { it.into() }

    override suspend fun indexSymbols() {
        inner.indexSymbols()
    }

    override suspend fun initStream(): Flow<Event> = inner.initStreamFlow().mapNotNull { it.into() }
}

class RustClientFactory(val url: String) : ClientFactory {

    private val _m = Mutex()
    private var client: RustClient? = null

    override suspend fun connect(): Client {
        client?.also {
            return it
        }

        _m.withLock {
            client?.also {
                return it
            }
            val c = RustClient(uniffi.client.Client.connect(url))
            client = c
            return c
        }
    }
}

fun uniffi.client.Client.serverCountFlow() = flow {
    serverCount().use { stream ->
        while (true) {
            stream.next()?.also { count -> emit(count) } ?: break
        }
    }
}

fun uniffi.client.Client.initStreamFlow() = flow {
    initStream().use { stream ->
        while (true) {
            stream.next()?.also { event -> emit(event) } ?: break
        }
    }
}

fun uniffi.client.Client.getOdexFilesFlow(pid: UInt) = flow {
    getOdexFiles(pid).use { stream ->
        while (true) {
            stream.next()?.also { file -> emit(file) } ?: break
        }
    }
}

fun uniffi.client.Client.getSoFilesFlow(pid: UInt) = flow {
    getOdexFiles(pid).use { stream ->
        while (true) {
            stream.next()?.also { file -> emit(file) } ?: break
        }
    }
}

fun uniffi.client.Client.getSymbolFlow(filePath: String) = flow {
    getSymbols(filePath).use { stream ->
        while (true) {
            stream.next()?.also { symbol -> emit(symbol) } ?: break
        }
    }
}
