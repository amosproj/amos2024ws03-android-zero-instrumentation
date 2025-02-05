// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.events

import de.amosproj3.ziofa.client.Event
import kotlinx.coroutines.flow.Flow

interface DataStreamProvider {

    /** Returns a flow of all [Event.VfsWrite] events for the given [pids]. */
    fun vfsWriteEvents(pids: List<UInt>?): Flow<Event.VfsWrite>

    /** Returns a flow of all [Event.SysSendmsg] events for the given [pids]. */
    fun sendMessageEvents(pids: List<UInt>?): Flow<Event.SysSendmsg>

    /** Returns a flow of all [Event.JniReferences] events for the given [pids]. */
    fun jniReferenceEvents(pids: List<UInt>?): Flow<Event.JniReferences>

    /** Returns a flow of all [Event.SysSigquit] events for the given [pids]. */
    fun sigquitEvents(pids: List<UInt>?): Flow<Event.SysSigquit>

    /** Returns a flow of all [Event.Gc] events for the given [pids]. */
    fun gcEvents(pids: List<UInt>?): Flow<Event.Gc>

    /** Returns a flow of all [Event.SysFdTracking] events for the given [pids]. */
    fun fileDescriptorTrackingEvents(pids: List<UInt>?): Flow<Event.SysFdTracking>
}
