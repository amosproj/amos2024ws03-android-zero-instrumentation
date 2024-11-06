import org.gradle.internal.extensions.stdlib.capitalized

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
}

fun generatedDir(subdir: String) = layout.buildDirectory.file("generated/source/uniffi/${subdir}/java").get().asFile

android {
    namespace = "de.amosproj3.ziofa.client"
    compileSdk = 35
    buildToolsVersion = "35.0.0"

    defaultConfig {
        minSdk = 33

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    sourceSets {
        getByName("debug") {
            kotlin.srcDir(generatedDir("debug"))
        }
        getByName("release") {
            kotlin.srcDir(generatedDir("release"))
        }
    }
}

dependencies {

    implementation(libs.jna) { artifact { type = "aar" } }
    testImplementation(libs.jna)

    implementation(libs.androidx.core.ktx)
    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

afterEvaluate {
    android.libraryVariants.forEach{ variant ->

        val task = tasks.register<Exec>("generate${variant.name}UniFFIBindings") {
            val cargoTask = tasks.getByName<CargoBuildTask>("cargoBuild${linuxTarget.capitalized()}")
            dependsOn(cargoTask)
            workingDir = rustDir
            commandLine(
                "cargo",
                "run",
                "--bin=uniffi-bindgen",
                "--features=uniffi",
                "--features=uniffi/cli",
                "generate",
                "--language=kotlin",
                "--library",
                layout.buildDirectory.file("rustJniLibs/${cargoTask.toolchain!!.folder}/lib${rustLibName}.so").get().asFile.path,
                "--out-dir", generatedDir(variant.name))
        }

        tasks.getByName("compile${variant.name.capitalized()}Kotlin").dependsOn(task)
    }
}

