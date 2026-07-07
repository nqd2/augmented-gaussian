# Augmented Gaussian GUI Design System

This file defines the GUI rules for the Augmented Gaussian calibration tool using the Viettel Telecom brand palette and the Astryx component library (`@astryxdesign/core`).

The app processes 3D Gaussian Splatting data into collision meshes and navmeshes for AR. The GUI needs dense controls, clear status, and a viewport that stays readable while users calibrate scale, floor direction, and bake settings.

## Tokens

### Typography

Use these typography tokens for all UI text:

- `font.family.primary = Roboto`
- `font.family.stack = Roboto, sans-serif`
- `font.size.base = 14px`
- `font.weight.base = 400`
- `font.lineHeight.base = 22.001px`
- `font.size.xs = 0px` for screen-reader-only text
- `font.size.sm = 14px` for secondary text, helper text, and button labels
- `font.size.md = 15px` for labels and input values
- `font.size.lg = 16px` for card subheadings and secondary action groups
- `font.size.xl = 17px` for section titles and panel headers
- `font.size.2xl = 18px` for workspace headers
- `font.size.3xl = 19px` for main panel status
- `font.size.4xl = 20px` for the app title

### Color

Use semantic color tokens in components. Avoid raw hex values in component code except where this file defines the token value.

- `color.text.primary = #2c2f31`
- `color.text.secondary = #333333`
- `color.surface.base = #000000`
- `color.surface.muted = #ffffff`
- `color.surface.raised = #d11313`
- `color.border.strong = #eaeaea`
- `color.border.focus = #d11313`

Viewport overlay colors:

- Floor calibration normal: `#d11313`
- Floor calibration points: `#ffd159`
- Scale calibration points: `#73b3ff`

### Spacing

Use only these spacing tokens:

- `space.1 = 2px`
- `space.2 = 4px`
- `space.3 = 5px`
- `space.4 = 8px`
- `space.5 = 9px`
- `space.6 = 10px`
- `space.7 = 11px`
- `space.8 = 12px`

### Radius, Shadow, Motion

- `radius.xs = 8px` for inputs and minor controls
- `radius.sm = 10px` for buttons and small cards
- `radius.md = 12px` for configuration cards
- `radius.lg = 16px` for panels and dialogs
- `radius.xl = 22px` for large banner segments
- `radius.2xl = 50px` for pill buttons and circular controls
- `motion.duration.instant = 200ms`
- `motion.duration.fast = 300ms`
- `motion.duration.normal = 500ms`

## Layout Rules

### Workspace

Use Astryx `XDSAppShell` or `XDSLayout` for the main shell.

- Give control panels and sidebars a `1px` `color.border.strong` border.
- Use `space.8` between panels.
- Stack panels vertically at widths `<= 1100px`.

### Configuration Panels

Use Astryx `Card`, `TextInput`, `NumberInput`, and `Selector` components for configuration groups.

- Cards use `1px solid color.border.strong`, `color.surface.muted`, and `space.6` padding.
- Text and number inputs use `font.size.md` for values.
- Active input borders transition over `motion.duration.instant`.
- Focus outlines use `color.border.focus`.
- Validation errors use `color.surface.raised`.
- Selectors support arrow-key navigation and `Enter`.
- Hide duplicated selector labels with `isLabelHidden` when the surrounding section already labels the field.

### Buttons

Use Astryx `Button`.

- `Bake Geometry` uses the `primary` variant, `color.surface.raised` background, `color.surface.muted` text, and `radius.2xl`.
- `Load Source`, `Cancel`, and `Browse` use `secondary` or icon-only variants.
- Hover transitions use `motion.duration.instant`.
- `:focus-visible` shows an outer `color.border.focus` outline.
- Disabled buttons use `opacity: 0.5`, `#eaeaea` background, and `pointer-events: none`.
- Loading buttons show their spinner inside the button boundary.

### Viewport

- Use `color.surface.base` for the viewport background.
- Render floor and scale points with the overlay colors listed above.
- Highlight the active axis with `color.surface.raised`.

### Log Console

- Use `ui-monospace, SFMono-Regular, monospace`.
- Use `font.size.sm`.
- Use `#eaeaea` for log text and `color.surface.raised` for errors.

## Accessibility

Target WCAG 2.2 AA.

- Normal text under 18pt keeps at least `4.5:1` contrast.
- Large text keeps at least `3:1` contrast.
- Controls, icons, and focus rings keep at least `3:1` contrast.
- Tab order moves from file inputs, calibration controls, viewport, point editor, then action buttons.
- Every focusable element shows a visible `:focus-visible` state.
- Do not remove outlines unless the replacement focus style stays visible in tests.
- Form fields use labels linked with `for` and `id`.

## Copy

- Use English for GUI graphics terms such as `Voxelize`, `NavMesh`, and `3D Gaussian Splatting`.
- Use active command labels: `Load Source`, `Bake Geometry`, `Cancel Job`, `Save ZIP`.
- Keep helper text short and specific.

## Implementation Limits

- Do not add one-off spacing values, margins, or font sizes.
- Do not position controls in ways that overflow at `320px` width.
- Do not use low-contrast secondary text.
- Do not ship unlabeled form fields.

## Naming

### Files and Directories

- React components and TypeScript classes use `PascalCase`: `Preview.tsx`, `PlayCanvasViewer.ts`.
- Utility files use `camelCase`: `calibration.ts`, `splatPreview.ts`.
- Stylesheets and assets use `kebab-case`: `styles.css`.
- Directories use lowercase or `kebab-case`: `domains/calibration`, `domains/viewer`.

### Code

- Variables and functions use `camelCase`: `isFinitePoint`, `handlePointerDown`.
- Static config constants use `UPPER_SNAKE_CASE` or `camelCase`: `STAGE_PROGRESS`, `defaultConfig`.
- Interfaces, types, and enums use `PascalCase`: `SourceMetadata`, `Bounds`, `PickMode`.
- Tauri IPC commands use Rust-style `snake_case`: `load_source`, `process_job`.

### CSS

- Custom classes use lowercase `kebab-case`: `.viewport-legend`, `.legend-item`.
- Custom classes do not use the `astryx-` prefix.

## QA Checklist

Before merging GUI code:

- [ ] Text and form-field contrast is at least `4.5:1`.
- [ ] Keyboard tests cover buttons, inputs, links, and cards with `Tab`, `Enter`, and `Space`.
- [ ] Focus outlines are visible and use `:focus-visible`.
- [ ] The app works at `320px` width without clipping or overlap.
- [ ] Spacing and typography use the tokens in this file.
- [ ] Hover and active animations use the motion tokens.
- [ ] CSS does not add raw color values outside token definitions.
