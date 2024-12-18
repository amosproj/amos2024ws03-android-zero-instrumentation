// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.about

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

const val PRODUCT_MISSION =
    "ZIOFA (Zero Instrumentation Observability for Android) aims to implement observability use " +
        "cases relevant to performance specified by our industry partner using eBPF. " +
        "Examples include tracing long-running blocking calls, leaking JNI indirect" +
        "references or signals like SIGKILL sent to processes, all without instrumenting" +
        " the observed application itself.\n" +
        "The eBPF programs are loaded and unloaded using a backend daemon running as root that " +
        "will collect metrics and send them to a client. For displaying these metrics to " +
        "the user, we are implementing an on-device UI that can display visualizations for" +
        " these use cases and allow for configuration of the enabled use cases, but using a " +
        "decoupled Client SDK so that future work may easily make the data accessible the " +
        "external processing."

/**
 * Screen containing information about the project. Might delete later if we need space for another
 * screen in the tab bar.
 */
@Composable
fun AboutScreen(modifier: Modifier = Modifier) {
    Box(modifier = modifier.padding(20.dp)) { Text(PRODUCT_MISSION) }
}
