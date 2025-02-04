// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import androidx.compose.ui.text.intl.Locale
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlin.time.toDuration
import kotlinx.datetime.Instant
import kotlinx.datetime.LocalDateTime
import kotlinx.datetime.TimeZone
import kotlinx.datetime.format
import kotlinx.datetime.format.byUnicodePattern
import kotlinx.datetime.toLocalDateTime

@Suppress("MagicNumber") // unit conversion
fun Double.nanosToMillis() = this / 1_000_000.0

fun ULong.nanosToSecondsStr() = this.toLong().nanosToSecondsStr()

@Suppress("MagicNumber") // unit conversion
fun Number.nanosToSecondsStr(): String {
    val locale = Locale.current.platformLocale
    return String.format(locale, "%.4f", this.toDouble() / 1_000_000_000)
}

fun ULong.nanosToMillisStr() = this.toLong().nanosToMillisStr()

@Suppress("MagicNumber") // unit conversion
fun Number.nanosToMillisStr(): String {
    val locale = Locale.current.platformLocale
    return String.format(locale, "%.4f", this.toDouble() / 1_000_000)
}

fun DropdownOption.Timeframe.toMillis() = this.amount.toDuration(this.unit).inWholeMilliseconds

fun DropdownOption.Timeframe.toSeconds() = this.amount.toDuration(this.unit).inWholeSeconds

@Suppress("MagicNumber")
fun Number.bytesToHumanReadableSize(): String {
    val bytes = this.toDouble()
    return when {
        bytes >= 1 shl 30 -> "%.1f GB".format(bytes / (1 shl 30))
        bytes >= 1 shl 20 -> "%.1f MB".format(bytes / (1 shl 20))
        bytes >= 1 shl 10 -> "%.0f kB".format(bytes / (1 shl 10))
        else -> "$this bytes"
    }
}

fun Instant.toHRString() =
    this.toLocalDateTime(TimeZone.currentSystemDefault())
        .format(LocalDateTime.Format { byUnicodePattern("yyyy:MM:dd HH:mm:ss.SSS") })

fun ULong.bytesToHumanReadableSize() = this.toDouble().bytesToHumanReadableSize()
