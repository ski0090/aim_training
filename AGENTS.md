# PUBG Aim Trainer (Mutant Edition)

## 프로젝트 개요
이 프로젝트는 **Rust**와 **Bevy 게임 엔진**을 사용하여 제작되는 PUBG 에임 연습 시뮬레이터입니다.
주요 목표는 **Mk47 Mutant** 총기의 특성(2점사, 반동)을 정확히 구현하여 사용자가 효과적으로 에임을 연습할 수 있도록 돕는 것입니다.
- Bevy 0.18.0 버전을 사용합니다.
- bevy_kira_audio를 사용합니다. (0.25 버전)

## 기술 스택
- **Language**: Rust
- **Game Engine**: Bevy

## 주요 기능
- **Mutant 시뮬레이션**: 2점사 메커니즘, 수직/수평 반동 패턴 구현
- **FPS 조작**: WASD 이동, 마우스 시점 변환, 줌(ADS) 기능
- **연습 모드**: 다양한 거리와 움직임을 가진 표적 맞추기

## 개발 로드맵
1. Bevy 프로젝트 설정 및 윈도우 생성
2. 플레이어 이동 및 카메라 제어 (FPS) (완료)
3. 총기 발사 로직 (Raycasting) 및 2점사 구현
4. 반동 시스템 적용
5. 표적 생성 및 피격 판정 시스템
6. UI (탄약, 점수, 크로스헤어) 구현

## 주요 실수 및 교훈 (Lessons Learned)

### 1. 오디오 볼륨 제어 (bevy_kira_audio)
- **증상**: `audio.set_volume(0.5)` 등을 호출해도 소리 크기가 변하지 않음.
- **원인**: `bevy_kira_audio`의 `set_volume` 메서드는 **선형적인 진폭(Linear Amplitude, 0.0 ~ 1.0)**이 아니라 **데시벨(Decibels)** 값을 받습니다.
  - `0.0`: 0dB (최대 볼륨/감쇠 없음)
  - `-60.0` (Decibels::SILENCE): 묵음
- **해결**: 선형 비율(0~100%)을 데시벨로 변환하여 사용해야 합니다.
  ```rust
  let amplitude = percentage / 100.0;
  let volume_db = if amplitude > 0.0 { 20.0 * amplitude.log10() } else { -60.0 };
  audio.set_volume(volume_db);
  ```
- **주의**: 오디오 인스턴스 생성 시(`.play(...).with_volume(1.0)`) 설정한 볼륨은 채널 볼륨과 곱해집니다. 채널 볼륨 제어를 우선시한다면 개별 인스턴스 볼륨 강제 설정을 피하는 것이 좋습니다.

### 2. Bevy API 및 버전 호환성
- **Timer API**: Bevy 최신 버전에서는 `Timer` 사용 시 API 변경이 있을 수 있으므로, 단순한 쿨타임 구현에는 `f32` 변수와 `time.delta_seconds()`를 사용하는 것이 더 안정적일 수 있습니다.
- **Query API**: Bevy 최신 버전(0.18.0)에서는 `get_single_mut()`가 제거되었습니다. 대신 `single_mut()`가 `Result`를 반환하도록 변경되었으므로, 안전한 접근을 위해 이를 사용해야 합니다. (예: `if let Ok(mut item) = query.single_mut() { ... }`)
- **Event API**: `EventWriter`와 `EventReader`가 제거되었습니다. 대신 `Writer`, `Reader`를 사용하거나, 시스템 간 통신에 `Resource`를 활용하는 것이 좋습니다.


