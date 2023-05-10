#!/usr/bin/env python3

import multiprocessing as mp
import pynput.keyboard
import pynput.mouse
import queue
import subprocess
import sys
import time

class Action:
    def __init__(self, actions, previous_event=None):
        self.actions = actions
        self.pe = previous_event
        
    def execute(self):
        for a in self.actions:
            a()

    def previous_event(self):
        return self.pe

def poweroff():
    print("not ready yet")
    # subprocess.Popen(['/usr/sbin/poweroff'],
    #                  stdout=subprocess.PIPE,
    #                  stderr=subprocess.PIPE)



def read_input(job_queue):
    keys = set(['up', 'down', 'left', 'right', 'select', 'exit', 'Fast forward', 'rewind'])
    actions = set(['pressed', 'released'])
    events = set([f'{a}: {k}' for a in actions for k in keys])
    events.add("power status changed from 'on' to 'standby'")

    try:
        for line in iter(sys.stdin.readline, b''):
            received_event = None
            for e in events:
                if e in line:
                    received_event = e
                    break

            if received_event is not None:
                job_queue.put(received_event)
    except KeyboardInterrupt:
        sys.stdout.flush()
        print("Received KeyboardInterrupt")
        pass
    job_queue.put(None)

def move_mouse(job_queue):
    mouse = pynput.mouse.Controller()
    keyboard = pynput.keyboard.Controller()

    movement_unit = 3
    acc = 1.0
    previous_event = None
    while True:
        time.sleep(0.01)
        
        event = None
        try:
            event = job_queue.get(block=False)
        except queue.Empty:
            event = previous_event

        if event is None:
            acc = 1.0
            continue

        jump = movement_unit*acc

        action_for_event = {
            # MOVEMENT
            "pressed: up":    Action([lambda: mouse.move(0, -jump)], event),
            "pressed: down":  Action([lambda: mouse.move(0, jump)], event),
            "pressed: left":  Action([lambda: mouse.move(-jump, 0)], event),
            "pressed: right": Action([lambda: mouse.move(jump, 0)], event),
            "released: up":    Action([lambda: ()]),
            "released: down":  Action([lambda: ()]),
            "released: left":  Action([lambda: ()]),
            "released: right": Action([lambda: ()]),
            # SCROLL
            "pressed: Fast forward": Action([lambda: mouse.scroll(0, -2)]),
            "pressed: rewind":       Action([lambda: mouse.scroll(0, 2)]),
            # SELECT
            "pressed: select": Action([lambda: mouse.press(pynput.mouse.Button.left)]),
            "released: select": Action([lambda: mouse.release(pynput.mouse.Button.left)]),
            "pressed: exit": Action([
                lambda: keyboard.press(pynput.keyboard.Key.alt_l),
                lambda: keyboard.press(pynput.keyboard.Key.left),
            ]),
            "released: exit": Action([
                lambda: keyboard.release(pynput.keyboard.Key.left),
                lambda: keyboard.release(pynput.keyboard.Key.alt_l),
            ]),
            "power status changed from 'on' to 'standby'": Action([
                poweroff()
            ]),
        }

        action = action_for_event.get(event, None)
        if action is not None:
            action.execute()
            previous_event = action.previous_event()

        acc *= 1.01

def main():
    job_queue = mp.JoinableQueue()

    mp.Process(target=move_mouse, args=(job_queue,), daemon=True).start()

    read_input(job_queue)

    job_queue.join()

main()
