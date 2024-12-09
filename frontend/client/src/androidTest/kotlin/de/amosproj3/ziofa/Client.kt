// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import androidx.test.ext.junit.runners.AndroidJUnit4
import de.amosproj3.ziofa.client.RustClientFactory
import kotlinx.coroutines.runBlocking
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class Test {
    @Test fun works(): Unit = runBlocking { RustClientFactory("https://google.com").connect() }
}
