package com.example.eqgateway.ui.chat

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.BugReport
import androidx.compose.material.icons.filled.DeleteSweep
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.eqgateway.model.EqState
import kotlinx.coroutines.flow.collectLatest

private val UserBubbleColor = Color(0xFF1A73E8)
private val AiBubbleColor = Color(0xFF2D2D2D)
private val UserTextColor = Color.White
private val AiTextColor = Color(0xFFE0E0E0)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(
    modifier: Modifier = Modifier,
    viewModel: ChatScreenViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }
    val listState = rememberLazyListState()

    // Auto-scroll to bottom when new messages arrive
    LaunchedEffect(state.messages.size) {
        if (state.messages.isNotEmpty()) {
            listState.animateScrollToItem(state.messages.size - 1)
        }
    }

    // Show errors as snackbar
    LaunchedEffect(state.errorMessage) {
        state.errorMessage?.let { msg ->
            snackbarHostState.showSnackbar(msg)
            viewModel.dismissError()
        }
    }

    Scaffold(
        modifier = modifier,
        snackbarHost = { SnackbarHost(snackbarHostState) },
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("EQ Gateway Chat", fontWeight = FontWeight.Bold)
                        Text(
                            text = state.engineStatus,
                            fontSize = 11.sp,
                            color = if (state.isInitialized) Color(0xFF81C784) else Color(0xFFFFB74D),
                        )
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.toggleDebugOverlay() }) {
                        Icon(
                            Icons.Default.BugReport,
                            contentDescription = "Toggle EQ State debug overlay",
                            tint = if (state.showDebugOverlay) Color(0xFFFFB74D)
                                   else MaterialTheme.colorScheme.onSurface,
                        )
                    }
                    IconButton(onClick = { viewModel.onWipeSession() }) {
                        Icon(
                            Icons.Default.DeleteSweep,
                            contentDescription = "Wipe session",
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                ),
            )
        },
        bottomBar = {
            ChatInputBar(
                inputText = state.inputText,
                isProcessing = state.isProcessing,
                isInitialized = state.isInitialized,
                onInputChanged = viewModel::onInputChanged,
                onSend = viewModel::onSendMessage,
            )
        },
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            // Debug overlay
            AnimatedVisibility(
                visible = state.showDebugOverlay,
                enter = fadeIn(),
                exit = fadeOut(),
            ) {
                DebugOverlay(state.messages)
            }

            // Message list
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
                contentPadding = PaddingValues(bottom = 8.dp),
            ) {
                items(state.messages, key = { it.id }) { message ->
                    MessageBubble(message)
                }

                // Loading indicator
                if (state.isProcessing) {
                    item {
                        Box(
                            modifier = Modifier.fillMaxWidth(),
                            contentAlignment = Alignment.CenterStart,
                        ) {
                            CircularProgressIndicator(
                                modifier = Modifier
                                    .padding(16.dp)
                                    .size(24.dp),
                                strokeWidth = 2.dp,
                            )
                        }
                    }
                }
            }
        }
    }
}

// ── Message Bubble ─────────────────────────────────────────

@Composable
fun MessageBubble(message: ChatMessage, modifier: Modifier = Modifier) {
    val isUser = message.isUser
    val bubbleColor = if (isUser) UserBubbleColor else AiBubbleColor
    val textColor = if (isUser) UserTextColor else AiTextColor
    val shape = if (isUser) {
        RoundedCornerShape(16.dp, 4.dp, 16.dp, 16.dp)
    } else {
        RoundedCornerShape(4.dp, 16.dp, 16.dp, 16.dp)
    }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        horizontalAlignment = if (isUser) Alignment.End else Alignment.Start,
    ) {
        Card(
            shape = shape,
            colors = CardDefaults.cardColors(containerColor = bubbleColor),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
            modifier = Modifier.widthIn(max = 320.dp),
        ) {
            Text(
                text = message.text,
                color = textColor,
                modifier = Modifier.padding(12.dp),
                lineHeight = 20.sp,
            )
        }

        // EQ State annotation for AI messages
        if (!isUser && message.eqState != null) {
            val eq = message.eqState
            Text(
                text = "affect: ${eq.affect.primary} | risk: ${eq.risk.level} | intent: ${eq.cognitive.intent}",
                fontSize = 9.sp,
                color = Color.Gray,
                fontStyle = FontStyle.Italic,
                modifier = Modifier.padding(start = 8.dp, top = 2.dp),
            )
        }
    }
}

// ── Input Bar ──────────────────────────────────────────────

@Composable
fun ChatInputBar(
    inputText: String,
    isProcessing: Boolean,
    isInitialized: Boolean,
    onInputChanged: (String) -> Unit,
    onSend: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surface)
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = inputText,
            onValueChange = onInputChanged,
            placeholder = { Text("Type a message...") },
            modifier = Modifier.weight(1f),
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { onSend() }),
            enabled = !isProcessing && isInitialized,
            shape = RoundedCornerShape(24.dp),
        )
        Spacer(modifier = Modifier.width(8.dp))
        Button(
            onClick = onSend,
            enabled = inputText.isNotBlank() && !isProcessing && isInitialized,
            shape = RoundedCornerShape(24.dp),
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp),
        ) {
            if (isProcessing) {
                CircularProgressIndicator(
                    modifier = Modifier.size(18.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary,
                )
            } else {
                Icon(Icons.Default.Send, contentDescription = "Send", modifier = Modifier.size(18.dp))
                Spacer(modifier = Modifier.width(4.dp))
                Text("Send")
            }
        }
    }
}

// ── Debug Overlay ──────────────────────────────────────────

@Composable
fun DebugOverlay(messages: List<ChatMessage>, modifier: Modifier = Modifier) {
    val latestEq = messages.lastOrNull { !it.isUser && it.eqState != null }?.eqState

    Column(
        modifier = modifier
            .fillMaxWidth()
            .background(Color(0xCC1A1A2E))
            .padding(12.dp),
    ) {
        Text(
            "EQ State Debug",
            fontWeight = FontWeight.Bold,
            color = Color(0xFFFFB74D),
            fontSize = 13.sp,
        )
        if (latestEq != null) {
            EqStateView(latestEq)
        } else {
            Text(
                "No EQ State data yet",
                color = Color.Gray,
                fontSize = 11.sp,
                modifier = Modifier.padding(top = 4.dp),
            )
        }
        Text(
            "• Total messages: ${messages.size}",
            color = Color.Gray,
            fontSize = 10.sp,
            modifier = Modifier.padding(top = 4.dp),
        )
    }
}

@Composable
fun EqStateView(eq: EqState, modifier: Modifier = Modifier) {
    Column(modifier = modifier.padding(top = 4.dp)) {
        DebugRow("Affect", "${eq.affect.primary} ${eq.affect.secondary.take(2)}")
        DebugRow("Valence/Arousal", "${"%.2f".format(eq.affect.valence)} / ${"%.2f".format(eq.affect.arousal)}")
        DebugRow("Confidence", "${"%.0f".format(eq.affect.confidence * 100)}%")
        DebugRow("Intent", eq.cognitive.intent)
        DebugRow("Topics", eq.cognitive.topics.take(3).joinToString(", "))
        DebugRow("Urgency", "${"%.2f".format(eq.cognitive.urgency)}")
        DebugRow("Risk Level", eq.risk.level.uppercase())
        if (eq.risk.requires_human) DebugRow("⚠ Requires HITL", "")
        if (eq.risk.blocked) DebugRow("🚫 BLOCKED", "")
    }
}

@Composable
fun DebugRow(label: String, value: String, modifier: Modifier = Modifier) {
    Row(modifier = modifier.padding(vertical = 1.dp)) {
        Text(
            text = "$label: ",
            color = Color(0xFF90CAF9),
            fontSize = 10.sp,
            fontWeight = FontWeight.Medium,
        )
        Text(
            text = value,
            color = Color(0xFFE0E0E0),
            fontSize = 10.sp,
        )
    }
}
