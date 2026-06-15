package com.example.eqgateway.ui.chat

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.example.eqgateway.data.ChatRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * UI state for the chat screen.
 */
data class ChatUiState(
    val messages: List<ChatMessage> = emptyList(),
    val inputText: String = "",
    val isProcessing: Boolean = false,
    val engineStatus: String = "Initializing...",
    val isInitialized: Boolean = false,
    val showDebugOverlay: Boolean = false,
    val errorMessage: String? = null,
)

class ChatScreenViewModel(
    private val repository: ChatRepository = ChatRepository(),
    private val modelPath: String = "models/phi-4-mini-instruct-q4_k_m.gguf",
) : ViewModel() {

    private val _uiState = MutableStateFlow(ChatUiState())
    val uiState: StateFlow<ChatUiState> = _uiState.asStateFlow()

    private var messageIdCounter = 0L

    init {
        initializeEngine()
    }

    private fun initializeEngine() {
        viewModelScope.launch {
            _uiState.update { it.copy(engineStatus = "Initializing EQ engine...") }
            val result = repository.initEngine(
                modelPath = modelPath,
                gpuLayers = 0u,
                contextSize = 4096u,
            )
            result.fold(
                onSuccess = {
                    _uiState.update {
                        it.copy(
                            isInitialized = true,
                            engineStatus = "EQ Engine ready",
                        )
                    }
                },
                onFailure = { error ->
                    _uiState.update {
                        it.copy(
                            engineStatus = "Init failed: ${error.message}",
                            errorMessage = "Engine initialization failed: ${error.message}",
                        )
                    }
                },
            )
        }
    }

    fun onInputChanged(text: String) {
        _uiState.update { it.copy(inputText = text) }
    }

    fun onSendMessage() {
        val currentInput = _uiState.value.inputText.trim()
        if (currentInput.isEmpty() || _uiState.value.isProcessing) return

        // Add user message
        val userMessage = ChatMessage(
            id = messageIdCounter++,
            text = currentInput,
            isUser = true,
        )
        _uiState.update {
            it.copy(
                messages = it.messages + userMessage,
                inputText = "",
                isProcessing = true,
                errorMessage = null,
            )
        }

        // Process through FFI
        viewModelScope.launch {
            val result = repository.processInput(currentInput)
            result.fold(
                onSuccess = { processResult ->
                    val aiMessage = ChatMessage(
                        id = messageIdCounter++,
                        text = processResult.responseText,
                        isUser = false,
                        eqState = processResult.eqState,
                    )
                    _uiState.update {
                        it.copy(
                            messages = it.messages + aiMessage,
                            isProcessing = false,
                        )
                    }
                },
                onFailure = { error ->
                    val errorMessage = ChatMessage(
                        id = messageIdCounter++,
                        text = "Error: ${error.message}",
                        isUser = false,
                    )
                    _uiState.update {
                        it.copy(
                            messages = it.messages + errorMessage,
                            isProcessing = false,
                            errorMessage = error.message,
                        )
                    }
                },
            )
        }
    }

    fun onWipeSession() {
        viewModelScope.launch {
            repository.wipeSessions()
            _uiState.update {
                it.copy(
                    messages = emptyList(),
                    engineStatus = "Sessions wiped",
                )
            }
        }
    }

    fun toggleDebugOverlay() {
        _uiState.update { it.copy(showDebugOverlay = !it.showDebugOverlay) }
    }

    fun dismissError() {
        _uiState.update { it.copy(errorMessage = null) }
    }
}
