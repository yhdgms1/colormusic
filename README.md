# Цветомузыка

Программа преобразует звук в цвет. Отправляет UDP пакеты на устройство, которое должно отображать цвета.
В файле `config.json` можно настроить адрес устройства и порт, на которое будут отправляться данные о цвете. Также можно настроить устройства, с которых будет слушаться звук.

Программу можно использовать и отдельно от Arduino, например, используя Raspberry Pi или ESP32.

## Подключение

О том как подключить Arduino к LED ленте можно прочесть в [этой статье](https://alexgyver.ru/lessons/arduino-rgb/). Помимо Arduino понадобиться Ethernet модуль.

## Arduino

В директории [firmware](/firmware/) храниться скетч и файлы к нему. 

## TODO

- Улучшить алгоритм преобразования звука в цвет