// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

/** How often the process list should be refreshed from the backend */
const val PROCESS_LIST_REFRESH_INTERVAL_MS = 1000L

/** The maximum number of datapoints to show on screen for Vico visualizations */
const val TIME_SERIES_SIZE_VICO = 20

/** The maximum number of datapoints to show on screen for YCharts visualizations */
const val TIME_SERIES_SIZE_YCHARTS = 15

/** Maximum number of histogram buckets to display, depends on screen size */
const val HISTOGRAM_BUCKETS = 10

/**
 * The static threshold that is set at the backend for each event. If an event takes longer than
 * this threshold, it is counted as blocking. Currently, all events that are below this threshold,
 * once set, will never arrive at the frontend. In the future, this hsould be configurable by the
 * user.
 */
const val DURATION_THRESHOLD = 32_000_000UL
