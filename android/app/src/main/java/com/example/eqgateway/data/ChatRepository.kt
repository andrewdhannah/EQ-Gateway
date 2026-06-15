package com.example.eqgateway.data

import com.example.eqgateway.model.EqState
import com.example.eqgateway.model.EqStateParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.UUID
import uniffi.eq_ffi.engineStatus
import uniffi.eq_ffi.initialize
import uniffi.eq_ffi.processUserInput
import uniffi.eq_ffi.wipeAllSessions

/**
 * Result of processing a user message through the EQ engine.
 */
data class ProcessResult(
    val responseText: String,
    val eqState: EqState,
)

/**
 * Repository bridging the Rust FFI EQ engine and the Android UI layer.
 * Call [initialize] once at app startup before sending messages.
 */
class ChatRepository {

    private var initialized = false
    private var initializationError: String? = null

    /**
     * Initialize the EQ engine with default settings.
     * Must be called from a background thread (e.g. via coroutine).
     * The .so is expected at jniLibs/<abi>/libeq_ffi.so.
     *
     * Model file: loaded from app's internal storage at "models/phi-4-mini-instruct-q4_k_m.gguf"
     */
    suspend fun initEngine(modelPath: String, gpuLayers: UInt = 0u, contextSize: UInt = 4096u): Result<Unit> =
        withContext(Dispatchers.IO) {
            try {
                initialize(modelPath, gpuLayers, contextSize)
                initialized = true
                initializationError = null
                Result.success(Unit)
            } catch (e: Exception) {
                initializationError = e.message ?: "Unknown init error"
                initialized = false
                Result.failure(e)
            }
        }

    /**
     * Process user input through the EQ engine (FFI call).
     * Returns both the EQ state and the adaptive response text.
     */
    suspend fun processInput(text: String): Result<ProcessResult> =
        withContext(Dispatchers.IO) {
            if (!initialized) {
                return@withContext Result.failure(
                    IllegalStateException("Engine not initialized: $initializationError")
                )
            }
            try {
                val sessionId = UUID.randomUUID().toString()
                val rawJson = processUserInput(text, sessionId)
                val eqState = EqStateParser.parse(rawJson)
                val responseText = buildAdaptiveReply(eqState, text)
                Result.success(ProcessResult(responseText = responseText, eqState = eqState))
            } catch (e: Exception) {
                Result.failure(e)
            }
        }

    /**
     * Wipe all session buffers (call when app backgrounds).
     */
    suspend fun wipeSessions(): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            wipeAllSessions()
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * Get engine diagnostic status.
     */
    suspend fun getEngineStatus(): Result<String> = withContext(Dispatchers.IO) {
        try {
            Result.success(engineStatus())
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    // ── Helpers ──────────────────────────────────────────────

    private fun buildAdaptiveReply(eqState: EqState, userText: String): String {
        val affect = eqState.affect.primary
        val risk = eqState.risk.level
        val intent = eqState.cognitive.intent

        // If the input is flagged as blocked, always refuse
        if (eqState.risk.blocked) {
            return "I'm not able to help with that request."
        }

        // Build a simple adaptive response based on the EQ state
        return buildString {
            when (intent) {
                "venting", "emotional_release" -> {
                    append("I hear you. It sounds like you're feeling ")
                    append(affect)
                    append(". I'm here if you need to talk it through.")
                }
                "question", "information_seeking" -> {
                    append("That's a great question. Let me think about it.\n\n")
                    append("You seem ")
                    append(affect)
                    append(" about this topic.")
                }
                "greeting" -> {
                    append("Hello! How can I help you today?")
                }
                "command" -> {
                    append("Let me work on that for you.")
                }
                else -> {
                    // Generic — mirror the affect lightly
                    if (affect in listOf("frustrated", "angry", "anxious", "sad")) {
                        append("I can sense this is important to you — ")
                        append(affect)
                        append(". Let me do my best to help.")
                    } else {
                        append("Thanks for sharing. I've noted your message.")
                    }
                }
            }

            // Append a routing annotation for debug visibility
            if (risk == "medium") {
                append("\n\n[This response requires human review]")
            }
        }
    }
}
