---
name: yamcli-calendar
description: "Яндекс Календарь: просмотр событий, создание и удаление встреч через CalDAV. Используй для задач с расписанием — что запланировано, когда встреча, создать событие, свободное время."
metadata:
  author: aikalinin (based on NextStat/yacli)
---

# yamcli calendar — Яндекс Календарь

Сначала прочитай `yamcli-shared` для настройки аккаунта и авторизации.

## Команды

### Список календарей

```
yamcli calendar calendars [--account ALIAS]
```

Возвращает доступные календари с именами и ID.

### Список событий

```
yamcli calendar events [ОТ] [ДО] [--calendar ИМЯ] [--limit N] [--account ALIAS]
```

ОТ и ДО — даты в формате `YYYY-MM-DD`. По умолчанию: 30 дней от сегодня, limit=20.

Пример — события на завтра:
```
yamcli calendar events 2026-03-14 2026-03-15
```

Пример — события на следующую неделю:
```
yamcli calendar events 2026-03-13 2026-03-20
```

### Создать событие

```
yamcli calendar create НАЗВАНИЕ НАЧАЛО КОНЕЦ [--calendar ИМЯ] [--description ТЕКСТ] [--location МЕСТО] [--account ALIAS]
```

НАЧАЛО и КОНЕЦ — ISO 8601: `2026-03-14T10:00:00`.

Пример:
```
yamcli calendar create "Синхронизация команды" 2026-03-14T10:00:00 2026-03-14T11:00:00 --description "Еженедельный стендап"
```

### Удалить событие

```
yamcli calendar delete UID [--calendar ИМЯ] [--account ALIAS]
```

UID берётся из поля `uid` в выводе `calendar events`.

## Типичный порядок работы

1. `yamcli calendar calendars` — посмотреть доступные календари
2. `yamcli calendar events` — посмотреть ближайшие события
3. `yamcli calendar create ...` — создать событие
