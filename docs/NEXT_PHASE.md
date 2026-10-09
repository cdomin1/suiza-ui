# Next phase

Requested by the user on 2026-10-09. These features are scheduled after the current core editor fixes, not prerequisites for the first GitHub push.

1. Advanced prototyping: auto-animate, component-state transitions, media/voice/gamepad triggers, and additional viewport/scroll controls. Basic flows, interactions, overlays and standalone HTML sharing have moved into the current release at the user’s request. Hosted links, access control and synchronized comments need a backend and are explicitly deferred.
2. Design systems: reusable components with instances, shared color and typography styles, and consistent updates across a document.
3. Real-time collaboration: simultaneous canvas editing, comments and feedback. Requires a shared document service, identity/access model and conflict handling; deployment and service choices are not yet specified.
4. Developer handoff: inspect dimensions and spacing, copy color/style tokens and code snippets, and export assets in a dedicated developer view.
5. AI Agents tab: a conversational panel with selected frames/layers attached as context, suggested workflows for tidying/polishing, generating designs, and creating plugins/shaders. Agent changes should be reviewable and undoable; model/provider configuration and execution permissions remain implementation decisions for this phase.

AI reference: [Figma — Get started with Figma AI](https://help.figma.com/hc/en-us/articles/24039793359767-Get-started-with-Figma-AI). The reference describes conversational canvas edits, applying a design system, feedback, and custom plugins/shaders. The user supplied a sidebar/chat layout reference on 2026-10-09.

The Assets/library browser is explicitly part of the **current release**, including iOS, macOS and Android/Material libraries. Reusable component instances and shared-style propagation remain next phase.

Current phase stays focused on reliable selection, movement, text/vector editing, menus, inspector controls, UI layout, platform integration and verification.
