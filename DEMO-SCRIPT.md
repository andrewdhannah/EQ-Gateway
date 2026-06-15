# Demo Script: EQ Gateway Evidence Walkthrough

This script is designed to demonstrate the "Privacy Firewall" thesis to a technical reviewer in under 3 minutes.

## Scenario: "The Workplace Conflict"
**User Input:** *"I'm really frustrated with my manager, Sarah. I feel like she's undermining my work and I'm considering quitting."*

### Step 1: Local Analysis (The Firewall)
- **Action:** User types the message in the Android App.
- **Observation:** The app calls the Rust engine. The **EQ State Debug Overlay** updates instantly.
- **Proof Point:** 
  - `Affect`: Frustrated/Angry.
  - `Intent`: Venting.
  - `Risk`: Medium (Workplace conflict).
  - `Privacy`: PII detected ("Sarah").

### Step 2: Deterministic Routing (The Governance)
- **Action:** User hits "Send".
- **Observation:** The Agent Gateway processes the message. Because the risk is `medium`, the response is **paused**.
- **Proof Point:** The UI shows "Awaiting human approval." The `receipt.json` records the status as `approval_required`.

### Step 3: Human-in-the-Loop (The Authority)
- **Action:** The reviewer opens the `/pending` endpoint (or an admin dashboard).
- **Observation:** They see the request, a redacted excerpt, and the EQ State. They click "Approve".
- **Proof Point:** The HITL gate releases the request.

### Step 4: Secure Cloud Interaction (The MCP Bridge)
- **Action:** The Cloud Agent receives the `EQ State` and the approval.
- **Observation:** The Agent generates a supportive response based on the *affect* and *intent*, not the *raw text*.
- **Proof Point:** The final response is delivered. The total trace shows: `User` $\rightarrow$ `Local Scan` $\rightarrow$ `Human Approval` $\rightarrow$ `Cloud Reasoning`.

### Step 5: The Audit Trail (The Proof)
- **Action:** Open the `receipts/` directory.
- **Observation:** Show the JSON receipt for the transaction.
- **Proof Point:** Confirm that `raw_text_left_device: false` (or `true` only after approval) and that the `input_hash` is used instead of the raw text.
