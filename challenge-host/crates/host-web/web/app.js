(function () {
  const state = {
    handshake: null,
    challenges: [],
    currentChallenge: null,
    currentLanguage: null,
    currentWorkspace: null,
    progress: null,
    dirty: false,
    drafts: {}
  };

  const brandingProfiles = {
    hazptr: {
      subtitle: "DSA hazing for data structures & algorithms mastery.",
      icon: "icons/hazptr-favicon.svg",
      mask: "icons/hazptr-mask.svg",
      maskColor: "#f56c3e",
      kicker: "~ hazptr web"
    },
    fzts: {
      subtitle:
        "Build increasingly complex Rust applications, from probability engines to distributed consensus, grounded in real-world finance, science, infrastructure, AI, and security use cases.",
      icon: "icons/fzts-favicon.svg",
      mask: "icons/fzts-mask.svg",
      maskColor: "#0c4a6e",
      kicker: "~ fzts play web"
    },
    default: {
      subtitle: "Adapter-driven tooling for shared practice.",
      icon: "icons/hazptr-favicon.svg",
      mask: "icons/hazptr-mask.svg",
      maskColor: "#171614",
      kicker: "~ challenge-host web"
    }
  };

  const gameTitle = document.getElementById("game-title");
  const gameSubtitle = document.getElementById("game-subtitle");
  const challengeList = document.getElementById("challenge-list");
  const progressSummary = document.getElementById("progress-summary");
  const languageCompletionMap = document.getElementById("language-completion-map");
  const activityHeatmap = document.getElementById("activity-heatmap");
  const challengeNotesCopy = document.getElementById("challenge-notes-copy");
  const introCopy = document.getElementById("intro-copy");
  const guideCopy = document.getElementById("guide-copy");
  const conceptList = document.getElementById("concept-list");
  const docList = document.getElementById("doc-list");
  const hintList = document.getElementById("hint-list");
  const hintStatus = document.getElementById("hint-status");
  const revealHintButton = document.getElementById("reveal-hint-button");
  const editor = document.getElementById("editor");
  const saveButton = document.getElementById("save-button");
  const testButton = document.getElementById("test-button");
  const resetButton = document.getElementById("reset-button");
  const output = document.getElementById("output");
  const outputStatus = document.getElementById("output-status");
  const workspaceLabel = document.getElementById("workspace-label");
  const workspaceStatus = document.getElementById("workspace-status");
  const fileLabel = document.getElementById("file-label");
  const serverNote = document.getElementById("server-note");
  const gameKicker = document.getElementById("game-kicker");
  const languageSelect = document.getElementById("language-select");

  if (window.location.protocol === "file:") {
    serverNote.innerHTML =
      "This page needs the local API server. Run <code>challenge-host web --adapter ...</code> and open the printed URL.";
    setBusyState(true);
    editor.value = "Run `challenge-host web --adapter ...` to use the shared browser host.";
    return;
  }

  boot().catch(showFatalError);

  async function boot() {
    setBusyState(true);
    workspaceStatus.textContent = "Loading adapter state…";
    bindEvents();

    const payload = await apiFetch("/api/bootstrap");
    applyBootstrap(payload);

    serverNote.textContent =
      "Local API is active. The adapter owns challenge logic; this host owns the shell.";
    setBusyState(false);
  }

  function applyBootstrap(payload) {
    state.handshake = payload.handshake;
    state.challenges = payload.challenges.challenges || [];
    state.progress = payload.progress || null;
    state.currentChallenge =
      payload.workspace.challenge_id || payload.challenges.current_challenge || null;
    state.currentLanguage =
      payload.workspace.language || payload.challenges.current_language || null;

    const profile =
      brandingProfiles[payload.handshake.game_id] || brandingProfiles.default;
    applyBranding(profile, payload.handshake.title);
    gameTitle.textContent = payload.handshake.title;

    renderChallengeList();
    renderProgress(state.progress);
    renderWorkspace(payload.workspace);
  }

  function applyBranding(profile, titleText) {
    if (gameKicker) {
      gameKicker.textContent = profile.kicker;
    }
    if (gameSubtitle) {
      gameSubtitle.textContent = profile.subtitle;
    }
    document.title = `${titleText} · challenge-host`;
    setFavicon(profile.icon, profile.mask, profile.maskColor);
  }

  function setFavicon(icon, mask, color) {
    const svgLink = document.getElementById("favicon-svg");
    if (svgLink) {
      svgLink.href = icon;
    }
    const maskLink = document.getElementById("favicon-mask");
    if (maskLink) {
      maskLink.href = mask;
      maskLink.setAttribute("color", color);
    }
  }

  function bindEvents() {
    editor.addEventListener("keydown", (event) => {
      if (
        event.key !== "Tab" ||
        event.shiftKey ||
        event.altKey ||
        event.ctrlKey ||
        event.metaKey
      ) {
        return;
      }

      event.preventDefault();
      editor.setRangeText("\t", editor.selectionStart, editor.selectionEnd, "end");
      editor.dispatchEvent(new Event("input", { bubbles: true }));
    });

    editor.addEventListener("input", () => {
      rememberCurrentDraft(editor.value);
      updateWorkspaceStatus();
    });

    saveButton.addEventListener("click", saveCurrentWorkspace);
    testButton.addEventListener("click", runTests);
    resetButton.addEventListener("click", resetWorkspace);
    revealHintButton.addEventListener("click", revealHint);
    if (languageSelect) {
      languageSelect.addEventListener("change", onLanguageChange);
    }
  }

  async function refreshChallengeList() {
    const payload = await apiFetch("/api/bootstrap");
    state.handshake = payload.handshake;
    state.challenges = payload.challenges.challenges || [];
    state.progress = payload.progress || null;
    renderChallengeList();
    renderProgress(state.progress);
  }

  async function loadWorkspace(challengeId, language) {
    snapshotDraftForCurrentWorkspace();
    setBusyState(true);
    workspaceStatus.classList.remove("is-error");
    workspaceStatus.textContent = "Loading workspace…";

    const params = new URLSearchParams({
      challenge_id: challengeId
    });
    if (language) {
      params.set("language", language);
    }

    const workspace = await apiFetch("/api/workspace?" + params.toString());
    state.currentChallenge = workspace.challenge_id;
    state.currentLanguage = workspace.language || null;
    state.currentWorkspace = workspace;

    renderChallengeList();
    renderWorkspace(workspace);
    setBusyState(false);
  }

  function renderChallengeList() {
    challengeList.innerHTML = "";

    state.challenges.forEach((challenge, index) => {
      const item = document.createElement("li");
      item.className = "challenge-item";

      const button = document.createElement("button");
      button.type = "button";
      button.className = "challenge-button";
      button.dataset.challengeIndex = String(index);

      if (challenge.id === state.currentChallenge) {
        button.classList.add("is-active");
      }

      const languages = challenge.available_languages.join(", ");
      button.innerHTML =
        '<span class="challenge-symbols">' +
        '<span class="challenge-kind-symbol">' +
        escapeHtml(symbolForChallenge(challenge)) +
        "</span>" +
        '<span class="challenge-status-symbol">' +
        statusSymbol(challenge.status) +
        "</span>" +
        "</span>" +
        '<span class="challenge-copy"><span class="challenge-title">' +
        escapeHtml(challenge.title) +
        '</span><span class="challenge-meta">' +
        escapeHtml((challenge.track || "trackless") + " · " + languages) +
        "</span></span>";

      button.addEventListener("click", async () => {
        if (challenge.id === state.currentChallenge) {
          focusEditor();
          return;
        }

        const language = challenge.available_languages[0] || null;
        await loadWorkspace(challenge.id, language);
      });
      button.addEventListener("keydown", (event) => {
        let targetIndex = null;

        if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
          targetIndex =
            index === 0 ? state.challenges.length - 1 : index - 1;
        } else if (event.key === "ArrowDown" || event.key === "ArrowRight") {
          targetIndex =
            index === state.challenges.length - 1 ? 0 : index + 1;
        } else if (event.key === "Home") {
          targetIndex = 0;
        } else if (event.key === "End") {
          targetIndex = state.challenges.length - 1;
        }

        if (targetIndex == null) {
          return;
        }

        event.preventDefault();
        focusChallengeButton(targetIndex);
      });

      item.appendChild(button);
      challengeList.appendChild(item);
    });
  }

  function renderWorkspace(workspace) {
    decorateWorkspaceEditor(workspace);
    state.currentWorkspace = workspace;
    state.currentChallenge = workspace.challenge_id;
    state.currentLanguage = workspace.language || null;

    const draft = state.drafts[workspaceKey(workspace.challenge_id, workspace.language)];
    workspaceLabel.textContent =
      workspace.title + (workspace.language ? " [" + workspace.language + "]" : "");
    fileLabel.textContent = workspace.editor.file_path;
    if (draft) {
      editor.value = draft.content;
      state.dirty = draft.content !== workspace.editor_base_content;
    } else {
      editor.value = workspace.editor_base_content;
      state.dirty = false;
    }
    challengeNotesCopy.textContent =
      workspace.challenge_notes || "No starter notes were embedded in this source file.";
    introCopy.textContent = workspace.intro || "No intro is available for this challenge.";
    guideCopy.textContent = workspace.guide || "No guide text is available for this challenge.";
    output.textContent = "";
    setOutputStatus("No test run yet.", "");

    renderList(
      conceptList,
      workspace.concepts,
      function (concept) {
        const item = document.createElement("li");
        item.textContent = concept;
        return item;
      },
      "No concepts listed."
    );

    renderList(
      docList,
      workspace.docs,
      function (doc) {
        const item = document.createElement("li");
        const link = document.createElement("a");
        link.href = doc.url;
        link.textContent = doc.label;
        link.target = "_blank";
        link.rel = "noreferrer";
        item.appendChild(link);
        return item;
      },
      "No docs listed."
    );

    hintList.innerHTML = "";
    if (!workspace.hints.length) {
      const item = document.createElement("li");
      item.textContent = "No hints revealed yet.";
      hintList.appendChild(item);
    } else {
      workspace.hints.forEach((hint) => {
        const item = document.createElement("li");
        const title = document.createElement("strong");
        const body = document.createElement("p");
        title.textContent = hint.label;
        body.textContent = hint.body;
        item.appendChild(title);
        item.appendChild(body);
        hintList.appendChild(item);
      });
    }

    hintStatus.textContent = renderHintStatus(workspace.hint_state);
    resetButton.disabled = !workspace.editor.can_reset;
    revealHintButton.disabled = !workspace.actions.can_reveal_hint;
    updateWorkspaceStatus();
    updateLanguageSelect();
    focusEditor();
  }

  function renderHintStatus(hintState) {
    if (!hintState) {
      return "No hint state returned by the adapter.";
    }

    if (hintState.mode === "full") {
      return "All hints are visible for this challenge.";
    }

    return (
      "Revealed " +
      hintState.revealed_count +
      " of " +
      hintState.total_count +
      " hints." +
      (hintState.next_cost != null ? " Next cost: " + hintState.next_cost + "." : "")
    );
  }

  function renderList(target, items, makeItem, emptyText) {
    target.innerHTML = "";

    if (!items.length) {
      const item = document.createElement("li");
      item.textContent = emptyText;
      target.appendChild(item);
      return;
    }

    items.forEach(function (entry) {
      target.appendChild(makeItem(entry));
    });
  }

  function renderProgress(progress) {
    if (!progressSummary || !languageCompletionMap || !activityHeatmap) {
      return;
    }

    if (!progress) {
      progressSummary.textContent = "No progress data returned by the adapter.";
      languageCompletionMap.innerHTML = "";
      activityHeatmap.innerHTML = "";
      return;
    }

    const activity = progress.activity || [];
    const activeDays = activity.filter(function (day) {
      return heatValue(day) > 0;
    }).length;
    const summaryParts = [
      progress.completed + "/" + progress.total + " complete"
    ];

    if (progress.streak_days != null) {
      summaryParts.push(progress.streak_days + " day streak");
    }
    if (progress.score != null) {
      summaryParts.push(progress.score + " score");
    }
    summaryParts.push(activeDays + " active days");
    progressSummary.textContent = summaryParts.join(" · ") + ".";

    renderLanguageProgress(progress.languages || [], progress);
    renderActivityHeatmap(activity);
  }

  function renderLanguageProgress(languages, progress) {
    languageCompletionMap.innerHTML = "";

    if (!languages.length) {
      languageCompletionMap.textContent =
        "No per-language completion data returned by the adapter.";
      return;
    }

    languages
      .slice()
      .sort(function (left, right) {
        if (right.total !== left.total) {
          return right.total - left.total;
        }
        return left.language.localeCompare(right.language);
      })
      .forEach(function (entry) {
        const row = document.createElement("article");
        row.className = "language-map-row";

        const head = document.createElement("div");
        head.className = "language-map-head";

        const label = document.createElement("strong");
        label.className = "language-map-label";
        label.textContent = entry.language;

        const count = document.createElement("span");
        count.className = "language-map-count";
        count.textContent = entry.completed + "/" + entry.total;

        head.appendChild(label);
        head.appendChild(count);
        row.appendChild(head);

        const strip = document.createElement("div");
        strip.className = "language-map-strip";

        const segmentCount = languageSegmentCount(entry.total, progress.total);
        const completedSegments =
          entry.total > 0
            ? Math.max(
                entry.completed > 0 ? 1 : 0,
                Math.round((entry.completed / entry.total) * segmentCount)
              )
            : 0;

        for (let index = 0; index < segmentCount; index += 1) {
          const cell = document.createElement("span");
          cell.className = "language-map-cell";
          if (index < completedSegments) {
            cell.classList.add("is-complete");
          }
          strip.appendChild(cell);
        }

        strip.title =
          entry.language +
          ": " +
          entry.completed +
          " of " +
          entry.total +
          " challenges complete";
        strip.setAttribute("aria-label", strip.title);
        row.appendChild(strip);

        languageCompletionMap.appendChild(row);
      });
  }

  function renderActivityHeatmap(activity) {
    activityHeatmap.innerHTML = "";

    const activityMap = new Map(
      activity.map(function (day) {
        return [day.date, day];
      })
    );
    const maxValue = activity.reduce(function (max, day) {
      return Math.max(max, heatValue(day));
    }, 0);
    const currentYear = new Date().getFullYear();
    const today = startOfDay(new Date());

    buildYearHeatmapDates(currentYear).forEach(function (date) {
      const key = dateKey(date);
      const day = activityMap.get(key) || { check_ins: 0, completed: 0 };
      const cell = document.createElement("span");
      const level = intensityLevel(day, maxValue);
      cell.className = "heatmap-cell is-level-" + level;

      if (day.completed > 0) {
        cell.classList.add("is-complete");
      }
      if (date.getFullYear() !== currentYear) {
        cell.classList.add("is-outside-year");
      }
      if (date > today) {
        cell.classList.add("is-future");
      }

      const checkIns = day.check_ins || 0;
      const completed = day.completed || 0;
      const parts = [];
      parts.push(checkIns + " check-ins");
      parts.push(completed + " solutions");
      cell.title = key + ": " + parts.join(", ");
      cell.setAttribute("aria-label", cell.title);
      activityHeatmap.appendChild(cell);
    });
  }

  async function saveCurrentWorkspace() {
    await performWorkspaceMutation("/api/save", "Saving…", async function (workspace) {
      replaceDraftWithWorkspace(workspace);
      renderWorkspace(workspace);
      workspaceStatus.textContent = "Saved current workspace.";
    });
  }

  async function resetWorkspace() {
    if (!state.currentWorkspace || !state.currentWorkspace.editor.can_reset) {
      return;
    }

    await performWorkspaceMutation(
      "/api/reset",
      "Resetting…",
      async function (workspace) {
        replaceDraftWithWorkspace(workspace);
        renderWorkspace(workspace);
        workspaceStatus.textContent = "Restored editor content from the adapter workspace.";
        output.textContent = "";
        setOutputStatus("Editor reset.", "");
      },
      false
    );
  }

  async function revealHint() {
    if (!state.currentWorkspace || !state.currentWorkspace.actions.can_reveal_hint) {
      return;
    }

    setBusyState(true);
    workspaceStatus.classList.remove("is-error");
    workspaceStatus.textContent = "Revealing next hint…";

    try {
      const workspace = await apiFetch("/api/reveal-hint", {
        method: "POST",
        body: JSON.stringify({
          challenge_id: state.currentChallenge,
          language: state.currentLanguage
        })
      });

      renderWorkspace(workspace);
      await refreshChallengeList();
      workspaceStatus.textContent = "Revealed the next hint.";
      setBusyState(false);
    } catch (error) {
      setBusyState(false);
      workspaceStatus.textContent = error.message;
      workspaceStatus.classList.add("is-error");
    }
  }

  async function runTests() {
    setBusyState(true);
    workspaceStatus.classList.remove("is-error");
    workspaceStatus.textContent = "Running tests…";
    setOutputStatus("Running tests…", "");

    try {
      const result = await apiFetch("/api/test", {
        method: "POST",
        body: JSON.stringify(currentRequestPayload())
      });

      state.dirty = false;
      await refreshChallengeList();
      await loadWorkspace(state.currentChallenge, state.currentLanguage);
      output.textContent = result.output || "(no output)";
      setOutputStatus(result.passed ? "PASS" : "FAIL", result.passed ? "is-pass" : "is-fail");
      workspaceStatus.textContent = result.passed
        ? "Tests passed."
        : "Tests failed. Inspect output on the right.";
      setBusyState(false);
    } catch (error) {
      setBusyState(false);
      output.textContent = error.message;
      setOutputStatus("Error", "is-fail");
      workspaceStatus.textContent = "Test run failed before completion.";
      workspaceStatus.classList.add("is-error");
    }
  }

  async function performWorkspaceMutation(path, label, onSuccess, includeContent) {
    setBusyState(true);
    workspaceStatus.classList.remove("is-error");
    workspaceStatus.textContent = label;

    try {
      const payload = includeContent === false
        ? {
            challenge_id: state.currentChallenge,
            language: state.currentLanguage
          }
        : currentRequestPayload();
      const workspace = await apiFetch(path, {
        method: "POST",
        body: JSON.stringify(payload)
      });

      await onSuccess(workspace);
      setBusyState(false);
      await refreshChallengeList();
      renderChallengeList();
    } catch (error) {
      setBusyState(false);
      workspaceStatus.textContent = error.message;
      workspaceStatus.classList.add("is-error");
      output.textContent = error.message;
      setOutputStatus("Error", "is-fail");
    }
  }

  function currentRequestPayload() {
    return {
      challenge_id: state.currentChallenge,
      language: state.currentLanguage,
      content: composeEditorContent(editor.value)
    };
  }

  function workspaceKey(challengeId, language) {
    return (challengeId || "") + "::" + (language || "");
  }

  function rememberCurrentDraft(content) {
    if (!state.currentWorkspace) {
      return;
    }

    state.drafts[workspaceKey(state.currentChallenge, state.currentLanguage)] = {
      content: content
    };
    state.dirty = content !== state.currentWorkspace.editor_base_content;
  }

  function replaceDraftWithWorkspace(workspace) {
    decorateWorkspaceEditor(workspace);
    state.drafts[workspaceKey(workspace.challenge_id, workspace.language)] = {
      content: workspace.editor_base_content
    };
    state.dirty = false;
  }

  function updateWorkspaceStatus() {
    if (!state.currentWorkspace) {
      return;
    }

    workspaceStatus.textContent = state.dirty
      ? "Unsaved changes."
      : "Loaded " + state.currentWorkspace.challenge_id + ".";
  }

  function updateLanguageSelect() {
    if (!languageSelect || !state.currentChallenge) {
      return;
    }
    const challenge = state.challenges.find(
      (entry) => entry.id === state.currentChallenge
    );
    const languages = challenge?.available_languages || [];

    languageSelect.innerHTML = languages
      .map((lang) => `<option value="${lang}">${lang}</option>`)
      .join("");
    if (languages.length) {
      const target = state.currentLanguage || languages[0];
      languageSelect.value = target;
      languageSelect.disabled = false;
    } else {
      languageSelect.disabled = true;
    }
  }

  function onLanguageChange() {
    if (!state.currentChallenge || !languageSelect) {
      return;
    }
    const selected = languageSelect.value || null;
    if (selected === state.currentLanguage) {
      return;
    }
    loadWorkspace(state.currentChallenge, selected);
  }

  function snapshotDraftForCurrentWorkspace() {
    if (!state.currentWorkspace) {
      return;
    }

    rememberCurrentDraft(editor.value);
  }

  function focusChallengeButton(index) {
    const buttons = challengeList.querySelectorAll(".challenge-button");
    const button = buttons[index];
    if (button) {
      button.focus();
    }
  }

  function focusEditor() {
    window.requestAnimationFrame(function () {
      editor.focus();
    });
  }

  function decorateWorkspaceEditor(workspace) {
    const split = splitLeadingCommentary(
      workspace.editor.content || "",
      workspace.editor.file_path || "",
      workspace.language || ""
    );
    workspace.editor_prefix = split.prefix;
    workspace.editor_base_content = split.editorContent;
    workspace.challenge_notes = split.commentary;
  }

  function splitLeadingCommentary(content, filePath, language) {
    const text = String(content || "");
    const lines = text.split("\n");
    const prefixes = leadingCommentaryPrefixes(filePath, language);
    const prefixLines = [];
    const noteLines = [];
    let index = 0;
    let sawCommentary = false;

    while (index < lines.length) {
      const line = lines[index];
      const trimmed = line.trim();

      if (!sawCommentary && trimmed === "") {
        prefixLines.push(line);
        noteLines.push("");
        index += 1;
        continue;
      }

      const stripped = stripLeadingCommentaryLine(trimmed, prefixes, index);
      if (stripped == null) {
        if (sawCommentary && trimmed === "") {
          prefixLines.push(line);
          noteLines.push("");
          index += 1;
        }
        break;
      }

      sawCommentary = true;
      prefixLines.push(line);
      noteLines.push(stripped);
      index += 1;
    }

    if (!sawCommentary) {
      return {
        prefix: "",
        commentary: "",
        editorContent: text
      };
    }

    return {
      prefix: prefixLines.join("\n") + "\n",
      commentary: noteLines.join("\n").replace(/^\n+|\s+$/g, ""),
      editorContent: lines.slice(index).join("\n").replace(/^\n+/, "")
    };
  }

  function leadingCommentaryPrefixes(filePath, language) {
    const lowerPath = String(filePath || "").toLowerCase();
    const lowerLanguage = String(language || "").toLowerCase();

    if (
      lowerLanguage === "python" ||
      lowerPath.endsWith(".py") ||
      lowerPath.endsWith(".sh") ||
      lowerPath.endsWith(".rb") ||
      lowerPath.endsWith(".toml") ||
      lowerPath.endsWith(".yaml") ||
      lowerPath.endsWith(".yml")
    ) {
      return ["#"];
    }

    if (lowerPath.endsWith(".sql")) {
      return ["--"];
    }

    return ["///", "//!", "//"];
  }

  function stripLeadingCommentaryLine(line, prefixes, index) {
    for (const prefix of prefixes) {
      if (prefix === "#" && index === 0 && line.startsWith("#!")) {
        return null;
      }

      if (!line.startsWith(prefix)) {
        continue;
      }

      return line.slice(prefix.length).replace(/^\s?/, "");
    }

    return null;
  }

  function composeEditorContent(editorContent) {
    if (!state.currentWorkspace) {
      return editorContent;
    }

    return (state.currentWorkspace.editor_prefix || "") + editorContent;
  }

  function heatValue(day) {
    return (day.check_ins || 0) + (day.completed || 0) * 2;
  }

  function symbolForChallenge(challenge) {
    const badge = (challenge.badges || []).find(function (entry) {
      return entry && entry.trim();
    });
    if (badge) {
      return badge.trim();
    }

    const gameId = state.handshake?.game_id || "";
    if (gameId === "hazptr") {
      return symbolForHazptrChallenge(challenge);
    }
    if (gameId === "fzts") {
      return symbolForFztsChallenge(challenge);
    }
    if (gameId === "compilerlings") {
      return symbolForCompilerlingsChallenge(challenge);
    }

    return "·";
  }

  function symbolForHazptrChallenge(challenge) {
    const track = String(challenge.track || "").toLowerCase();

    if (track.includes("sorting")) {
      return "⇅";
    }
    if (track.includes("tree")) {
      return "Y";
    }
    if (track.includes("graph")) {
      return "◫";
    }
    if (track.includes("hash")) {
      return "#";
    }
    if (track.includes("heap")) {
      return "△";
    }
    if (track.includes("string")) {
      return "¶";
    }
    if (track.includes("dynamic")) {
      return "≋";
    }

    return "•";
  }

  function symbolForFztsChallenge(challenge) {
    const id = String(challenge.id || "");
    const match = id.match(/^(\d+)/);
    const number = match ? Number(match[1]) : NaN;

    if (number >= 1 && number <= 5) {
      return "Σ";
    }
    if (number >= 6 && number <= 8) {
      return "◇";
    }
    if (number >= 9 && number <= 12) {
      return "⚙";
    }
    if (number >= 13 && number <= 17) {
      return "▣";
    }
    if (number >= 18 && number <= 23) {
      return "⇄";
    }
    if (number >= 24 && number <= 29) {
      return "◎";
    }

    return "•";
  }

  function symbolForCompilerlingsChallenge(challenge) {
    const text = (String(challenge.id || "") + " " + String(challenge.title || "")).toLowerCase();

    if (text.includes("lexer") || text.includes("token")) {
      return "◌";
    }
    if (text.includes("parser") || text.includes("ast")) {
      return "⊏";
    }
    if (text.includes("type") || text.includes("semantic")) {
      return "⊢";
    }
    if (text.includes("ir") || text.includes("lower")) {
      return "λ";
    }
    if (text.includes("codegen") || text.includes("emit")) {
      return "⚡";
    }
    if (text.includes("vm") || text.includes("bytecode")) {
      return "▸";
    }

    return "•";
  }

  function intensityLevel(day, maxValue) {
    const value = heatValue(day);
    if (value <= 0 || maxValue <= 0) {
      return 0;
    }
    if (value >= maxValue) {
      return 4;
    }

    return Math.max(1, Math.ceil((value / maxValue) * 4));
  }

  function languageSegmentCount(total, fallbackTotal) {
    if (total > 0 && total <= 24) {
      return total;
    }

    const basis = total || fallbackTotal || 1;
    return Math.max(8, Math.min(24, basis));
  }

  function buildYearHeatmapDates(year) {
    const start = startOfWeek(new Date(year, 0, 1));
    const end = endOfWeek(new Date(year, 11, 31));
    const dates = [];

    for (let day = new Date(start); day <= end; day.setDate(day.getDate() + 1)) {
      dates.push(new Date(day));
    }

    return dates;
  }

  function startOfDay(date) {
    const value = new Date(date);
    value.setHours(0, 0, 0, 0);
    return value;
  }

  function startOfWeek(date) {
    const value = startOfDay(date);
    value.setDate(value.getDate() - value.getDay());
    return value;
  }

  function endOfWeek(date) {
    const value = startOfDay(date);
    value.setDate(value.getDate() + (6 - value.getDay()));
    return value;
  }

  function dateKey(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return year + "-" + month + "-" + day;
  }

  function setBusyState(busy) {
    saveButton.disabled = busy;
    testButton.disabled = busy;
    revealHintButton.disabled =
      busy ||
      !state.currentWorkspace ||
      !state.currentWorkspace.actions.can_reveal_hint;

    if (state.currentWorkspace) {
      resetButton.disabled = busy || !state.currentWorkspace.editor.can_reset;
    } else {
      resetButton.disabled = busy;
    }
  }

  function setOutputStatus(label, className) {
    outputStatus.textContent = label;
    outputStatus.className = "output-status";
    if (className) {
      outputStatus.classList.add(className);
    }
  }

  async function apiFetch(path, options) {
    const response = await fetch(path, {
      headers: {
        "Content-Type": "application/json"
      },
      ...options
    });

    const text = await response.text();
    let payload = null;

    if (text) {
      try {
        payload = JSON.parse(text);
      } catch (_error) {
        throw new Error("Received invalid JSON from the local server.");
      }
    }

    if (!response.ok) {
      throw new Error(payload && payload.error ? payload.error : "Request failed.");
    }

    return payload;
  }

  function showFatalError(error) {
    workspaceLabel.textContent = "challenge-host";
    workspaceStatus.textContent = "Unable to load workspace.";
    workspaceStatus.classList.add("is-error");
    output.textContent = error.message;
    setOutputStatus("Error", "is-fail");
    editor.value = "";
    setBusyState(true);
  }

  function statusSymbol(status) {
    if (status === "complete") {
      return "✓";
    }
    if (status === "in_progress") {
      return "·";
    }
    return "○";
  }

  function escapeHtml(text) {
    return String(text)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }
})();
