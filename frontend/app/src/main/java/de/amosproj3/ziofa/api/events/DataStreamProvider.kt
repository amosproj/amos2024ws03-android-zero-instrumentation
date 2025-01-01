// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.events

import de.amosproj3.ziofa.client.Event
import kotlinx.coroutines.flow.Flow

interface DataStreamProvider {

    fun vfsWriteEvents(pids: List<UInt>?): Flow<Event.VfsWrite>

    fun sendMessageEvents(pids: List<UInt>?): Flow<Event.SysSendmsg>

    fun jniReferenceEvents(pids: List<UInt>?): Flow<Event.JniReferences>
}
