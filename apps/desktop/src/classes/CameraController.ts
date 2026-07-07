import * as pc from 'playcanvas';
import { boundsPointToPoint3, type Bounds } from '../domains/calibration';

type PointerState = {
  clientX: number;
  clientY: number;
  button: number;
  shiftKey: boolean;
  pointerType: string;
};

export class CameraController {
  private cameraEntity: pc.Entity;
  private canvas: HTMLCanvasElement;
  private enabled = true;

  // First person / character POV camera state
  private currentPosition = new pc.Vec3(0, 0, 5);
  private targetPosition = new pc.Vec3(0, 0, 5);

  private currentYaw = 0; // Horizontal rotation in degrees
  private targetYaw = 0;
  private currentPitch = 0; // Vertical rotation in degrees
  private targetPitch = 0;

  // Angle limits (pitch) to avoid going upside down
  private readonly minPitch = -89.0;
  private readonly maxPitch = 89.0;

  private readonly rotateSpeed = 0.15; // Degrees per pixel drag
  private readonly dampingSpeed = 15.0;
  private flySpeed = 5.0; // Dynamically set based on fitBounds
  private readonly fastFlyMultiplier = 3.0;

  private activePointers = new Map<number, PointerState>();
  private isDragging = false;
  private pressedKeys = new Set<string>();

  constructor(cameraEntity: pc.Entity, canvas: HTMLCanvasElement) {
    this.cameraEntity = cameraEntity;
    this.canvas = canvas;

    this.currentPosition.copy(this.cameraEntity.getPosition());
    this.targetPosition.copy(this.currentPosition);

    const euler = this.cameraEntity.getEulerAngles();
    this.currentPitch = this.targetPitch = euler.x;
    this.currentYaw = this.targetYaw = euler.y;

    this.setupListeners();
  }

  private setupListeners() {
    this.canvas.style.touchAction = 'none';

    this.canvas.addEventListener('contextmenu', this.handleContextMenu);
    this.canvas.addEventListener('pointerdown', this.handlePointerDown);
    this.canvas.addEventListener('pointermove', this.handlePointerMove);
    this.canvas.addEventListener('pointerup', this.handlePointerUp);
    this.canvas.addEventListener('pointercancel', this.handlePointerUp);
    this.canvas.addEventListener('wheel', this.handleWheel, { passive: false });
    window.addEventListener('keydown', this.handleKeyDown);
    window.addEventListener('keyup', this.handleKeyUp);
    window.addEventListener('blur', this.handleWindowBlur);
  }

  public destroy() {
    this.canvas.removeEventListener('contextmenu', this.handleContextMenu);
    this.canvas.removeEventListener('pointerdown', this.handlePointerDown);
    this.canvas.removeEventListener('pointermove', this.handlePointerMove);
    this.canvas.removeEventListener('pointerup', this.handlePointerUp);
    this.canvas.removeEventListener('pointercancel', this.handlePointerUp);
    this.canvas.removeEventListener('wheel', this.handleWheel);
    window.removeEventListener('keydown', this.handleKeyDown);
    window.removeEventListener('keyup', this.handleKeyUp);
    window.removeEventListener('blur', this.handleWindowBlur);
  }

  public setEnabled(enabled: boolean) {
    this.enabled = enabled;
    if (!enabled) this.resetPointerGesture();
  }

  private handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
  };

  private handlePointerDown = (e: PointerEvent) => {
    if (!this.enabled) return;

    e.preventDefault();
    this.canvas.setPointerCapture(e.pointerId);
    this.activePointers.set(e.pointerId, {
      clientX: e.clientX,
      clientY: e.clientY,
      button: e.button,
      shiftKey: e.shiftKey,
      pointerType: e.pointerType,
    });

    if (this.activePointers.size === 1) {
      this.isDragging = true;
    }
  };

  private handlePointerMove = (e: PointerEvent) => {
    if (!this.enabled || !this.activePointers.has(e.pointerId)) return;

    e.preventDefault();

    const prev = this.activePointers.get(e.pointerId)!;
    const dx = e.clientX - prev.clientX;
    const dy = e.clientY - prev.clientY;

    this.activePointers.set(e.pointerId, {
      clientX: e.clientX,
      clientY: e.clientY,
      button: prev.button,
      shiftKey: e.shiftKey,
      pointerType: prev.pointerType,
    });

    if (this.isDragging && this.activePointers.size === 1) {
      this.targetYaw -= dx * this.rotateSpeed;
      this.targetPitch = this.clamp(this.targetPitch - dy * this.rotateSpeed, this.minPitch, this.maxPitch);
    }
  };

  private handlePointerUp = (e: PointerEvent) => {
    if (this.activePointers.has(e.pointerId)) {
      try {
        this.canvas.releasePointerCapture(e.pointerId);
      } catch {}
    }

    this.activePointers.delete(e.pointerId);

    if (this.activePointers.size === 0) {
      this.resetPointerGesture();
      return;
    }
  };

  private handleWheel = (e: WheelEvent) => {
    if (!this.enabled) return;

    e.preventDefault();

    // Wheel moves forward/backward
    const scrollSpeed = 0.05;
    const forward = this.cameraEntity.forward;
    const move = forward.clone().mulScalar(-e.deltaY * scrollSpeed * this.flySpeed * 0.1);
    this.targetPosition.add(move);
  };

  private handleKeyDown = (e: KeyboardEvent) => {
    const target = e.target as HTMLElement;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
      return;
    }
    const key = movementKey(e.code);
    if (!key) return;
    e.preventDefault();
    this.pressedKeys.add(key);
  };

  private handleKeyUp = (e: KeyboardEvent) => {
    const target = e.target as HTMLElement;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
      return;
    }
    const key = movementKey(e.code);
    if (!key) return;
    this.pressedKeys.delete(key);
  };

  private handleWindowBlur = () => {
    this.pressedKeys.clear();
  };

  public fitBounds(bounds?: Bounds, immediate = true) {
    const min = bounds ? boundsPointToPoint3(bounds.min) : [-2, -1, -2];
    const max = bounds ? boundsPointToPoint3(bounds.max) : [2, 2, 2];

    const center = new pc.Vec3(
      (min[0] + max[0]) * 0.5,
      (min[1] + max[1]) * 0.5,
      (min[2] + max[2]) * 0.5
    );

    const size = new pc.Vec3(max[0] - min[0], max[1] - min[1], max[2] - min[2]);
    const boundingRadius = Math.max(size.length() * 0.5, 1.0);
    const fov = this.cameraEntity.camera?.fov ?? 45;
    const fitRadius = boundingRadius / Math.sin(((fov * Math.PI) / 180) * 0.5);

    this.flySpeed = Math.max(boundingRadius * 0.4, 1.0);

    const defaultOffset = new pc.Vec3(1.2, 0.8, 1.5).normalize().mulScalar(fitRadius);
    const startPos = center.clone().add(defaultOffset);

    this.targetPosition.copy(startPos);

    this.cameraEntity.setPosition(startPos);
    this.cameraEntity.lookAt(center);

    const euler = this.cameraEntity.getEulerAngles();
    this.targetPitch = euler.x;
    this.targetYaw = euler.y;

    if (immediate) {
      this.currentPosition.copy(this.targetPosition);
      this.currentPitch = this.targetPitch;
      this.currentYaw = this.targetYaw;
      this.cameraEntity.setPosition(this.currentPosition);
      this.cameraEntity.setEulerAngles(this.currentPitch, this.currentYaw, 0);
    }
  }

  public resetView() {
    this.fitBounds(undefined, true);
  }

  public update(dt: number) {
    if (!this.enabled) return;

    this.updateFly(dt);

    const damping = this.damping(dt);

    this.currentYaw = this.lerpAngle(this.currentYaw, this.targetYaw, damping);
    this.currentPitch += (this.targetPitch - this.currentPitch) * damping;

    this.cameraEntity.setEulerAngles(this.currentPitch, this.currentYaw, 0);

    this.currentPosition.lerp(this.currentPosition, this.targetPosition, damping);
    this.cameraEntity.setPosition(this.currentPosition);
  }

  private updateFly(dt: number) {
    if (this.pressedKeys.size === 0) return;

    const forwardAmount = axis(this.pressedKeys, 'KeyW', 'KeyS') + axis(this.pressedKeys, 'ArrowUp', 'ArrowDown');
    const rightAmount = axis(this.pressedKeys, 'KeyD', 'KeyA') + axis(this.pressedKeys, 'ArrowRight', 'ArrowLeft');
    const upAmount = axis(this.pressedKeys, 'KeyE', 'KeyQ');
    if (!forwardAmount && !rightAmount && !upAmount) return;

    const forward = this.cameraEntity.forward.clone();
    const right = this.cameraEntity.right.clone();
    const up = new pc.Vec3(0, 1, 0);

    const move = new pc.Vec3()
      .addScaled(forward, forwardAmount)
      .addScaled(right, rightAmount)
      .addScaled(up, upAmount);

    if (move.lengthSq() <= 1e-8) return;

    const fast = this.pressedKeys.has('ShiftLeft') || this.pressedKeys.has('ShiftRight');
    const speed = this.flySpeed * (fast ? this.fastFlyMultiplier : 1);
    move.normalize().mulScalar(speed * Math.max(dt, 0));
    this.targetPosition.add(move);
  }

  private resetPointerGesture() {
    this.isDragging = false;
    this.activePointers.clear();
  }

  private damping(dt: number) {
    return 1.0 - Math.exp(-this.dampingSpeed * Math.max(dt, 0));
  }

  private lerpAngle(from: number, to: number, t: number) {
    let delta = (to - from) % 360;
    if (delta > 180) delta -= 360;
    if (delta < -180) delta += 360;
    return from + delta * t;
  }

  private clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value));
  }
}

function movementKey(code: string) {
  return [
    'KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyQ', 'KeyE',
    'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
    'ShiftLeft', 'ShiftRight'
  ].includes(code)
    ? code
    : null;
}

function axis(keys: Set<string>, positive: string, negative: string) {
  return (keys.has(positive) ? 1 : 0) - (keys.has(negative) ? 1 : 0);
}
