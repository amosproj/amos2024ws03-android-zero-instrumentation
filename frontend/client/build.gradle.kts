// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.org.cyclonedx.bom)
}

android {
    namespace = "de.amosproj3.ziofa.client"
    compileSdk = 35
    buildToolsVersion = "35.0.0"
    ndkVersion = "28.0.12433566"

    defaultConfig {
        minSdk = 33

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions { jvmTarget = "11" }

    // Flavors and build types need to at least contain the ones of the app
    flavorDimensions.add("version")
    productFlavors {
        create("real") { dimension = "version" }
        create("mock") { dimension = "version" }
    }
}

dependencies {
    val realImplementation by configurations.getting
    val testRealImplementation by configurations.getting
    val testRealDebugImplementation by configurations.creating
    val testRealReleaseImplementation by configurations.creating

    realImplementation(project(":bindings"))
    testRealImplementation(project(":bindings", "desktopLibs"))
    testRealDebugImplementation(project(":bindings", "desktopDebugLibs"))
    testRealReleaseImplementation(project(":bindings", "desktopReleaseLibs"))

    implementation(libs.kotlinx.coroutines.android)
    testImplementation(libs.junit)

    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

tasks.cyclonedxBom {
    setSchemaVersion("1.5")
    setIncludeConfigs(listOf("runtimeClasspath"))
    setOutputName("bom")
    setOutputFormat("json")
    setDestination(project.file("build/reports"))
    setIncludeBomSerialNumber(false)
    setIncludeLicenseText(true)
    setIncludeMetadataResolution(true)
}
