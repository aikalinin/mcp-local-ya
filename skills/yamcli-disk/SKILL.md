---
name: yamcli-disk
description: "Яндекс Диск: файлы, папки, загрузка и скачивание через REST API. Используй для задач с файлами — посмотреть что на диске, загрузить документ, скачать файл, создать папку, публичные ссылки."
metadata:
  author: aikalinin (based on NextStat/yacli)
---

# yamcli disk — Яндекс Диск

Сначала прочитай `yamcli-shared` для настройки аккаунта и авторизации.

## Команды

### Информация о диске

```
yamcli disk info [--account ALIAS]
```

Возвращает квоту: общий объём, занято, свободно.

### Список файлов

```
yamcli disk list [ПУТЬ] [--limit N] [--offset N] [--account ALIAS]
```

ПУТЬ использует префикс `disk:/`. По умолчанию: path=disk:/, limit=100, offset=0.

Пример:
```
yamcli disk list disk:/Документы --limit 50
```

### Создать папку

```
yamcli disk mkdir ПУТЬ [--account ALIAS]
```

Пример:
```
yamcli disk mkdir disk:/Документы/отчёты
```

### Загрузить файл

```
yamcli disk upload ФАЙЛ ПУТЬ [--overwrite] [--account ALIAS]
```

ФАЙЛ — локальный путь. ПУТЬ — место назначения на диске.

Пример:
```
yamcli disk upload ./отчёт.pdf disk:/Документы/отчёт.pdf --overwrite
```

### Публичные файлы

```
yamcli disk public show --public-key URL [--path ПУТЬ] [--account ALIAS]
yamcli disk public download --public-key URL --output ФАЙЛ [--path ПУТЬ] [--force] [--account ALIAS]
```

Доступ к расшаренным файлам по публичной ссылке.

## Типичный порядок работы

1. `yamcli disk info` — проверить свободное место
2. `yamcli disk list disk:/путь` — просмотреть файлы
3. `yamcli disk upload ./файл disk:/путь/файл` — загрузить файл
