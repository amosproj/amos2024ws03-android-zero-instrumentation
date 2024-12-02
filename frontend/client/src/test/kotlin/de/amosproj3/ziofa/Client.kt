// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.runBlocking
import org.junit.Test

class Test {
    @Test
    fun works(): Unit = runBlocking {
        val client = RustClientFactory("https://google.com").connect()
    }
}
