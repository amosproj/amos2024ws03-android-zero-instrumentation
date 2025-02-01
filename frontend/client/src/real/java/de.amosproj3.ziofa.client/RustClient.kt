// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.client.jniMethodNameFromI32
import uniffi.client.sysFdActionFromI32
import uniffi.shared.Cmd
import uniffi.shared.EventData
import uniffi.shared.EventType
import uniffi.shared.Filter
import uniffi.shared.JniMethodName
import uniffi.shared.MissingBehavior
import uniffi.shared.SysFdAction
import uniffi.shared.UInt32Filter

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
        vfsWrite = writeConfig?.let { VfsWriteConfig(pids = it.filter.toPidList()) },
        sysSendmsg =
            blockingConfig?.let { config ->
                config.filter.toPidList().let {
                    SysSendmsgConfig(
                        entries = it.associateWith { config.threshold ?: 32_000_000UL }
                    )
                }
            },
        uprobes =
            uprobeConfigs.map {
                UprobeConfig(
                    fnName = it.fnName,
                    offset = it.offset,
                    target = it.target,
                    pid = it.pid,
                )
            },
        jniReferences = jniReferencesConfig?.let { JniReferencesConfig(it.filter.toPidList()) },
        sysSigquit = signalConfig?.let { SysSigquitConfig(pids = it.filter.toPidList()) },
        gc = garbageCollectConfig?.let { GcConfig(it.filter.toPidList().toSet()) },
        sysFdTracking =
            fileDescriptorChangeConfig?.let { SysFdTrackingConfig(it.filter.toPidList()) },
    )

private fun Configuration.into() =
    uniffi.shared.Configuration(
        writeConfig = vfsWrite?.let { uniffi.shared.WriteConfig(it.pids.toPidFilter()) },
        blockingConfig =
            sysSendmsg?.let {
                uniffi.shared.BlockingConfig(
                    it.entries.keys.toPidFilter(),
                    it.entries.values.firstOrNull() ?: 32_000_000U,
                )
            },
        uprobeConfigs =
            uprobes.map {
                uniffi.shared.UprobeConfig(
                    fnName = it.fnName,
                    offset = it.offset,
                    target = it.target,
                    pid = it.pid,
                )
            },
        jniReferencesConfig =
            jniReferences?.let { uniffi.shared.JniReferencesConfig(it.pids.toPidFilter()) },
        signalConfig = sysSigquit?.let { uniffi.shared.SignalConfig(it.pids.toPidFilter()) },
        garbageCollectConfig =
            gc?.let { uniffi.shared.GarbageCollectConfig(it.pids.toPidFilter()) },
        fileDescriptorChangeConfig =
            sysFdTracking?.let { uniffi.shared.FileDescriptorChangeConfig(it.pids.toPidFilter()) },
    )

private fun Filter?.toPidList() = this?.pidFilter?.match ?: listOf()

private fun Collection<UInt>.toPidFilter() =
    Filter(
        pidFilter =
            UInt32Filter(
                missingBehavior = MissingBehavior.NOT_MATCH.value,
                match = this.toList(),
                notMatch = listOf(),
            ),
        commFilter = null,
        exePathFilter = null,
        cmdlineFilter = null,
    )

private fun uniffi.shared.Symbol.into() = Symbol(method, offset)

class RustClient(private val inner: uniffi.client.Client) : Client {
    override suspend fun listProcesses(): List<Process> = inner.listProcesses().map { it.into() }

    override suspend fun getConfiguration(): Configuration = inner.getConfiguration().into()

    // TODO remove the workarounds
    override suspend fun setConfiguration(configuration: Configuration) =
        inner.setConfiguration(configuration.into())

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

fun uniffi.client.Client.initStreamFlow() = flow {
    initStream().use { stream ->
        while (true) {
            stream.next()?.also { event -> emit(event) } ?: break
        }
    }
}
