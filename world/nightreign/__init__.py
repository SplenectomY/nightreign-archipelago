        roster = [n for n in self._nightfarers() if n != "Nightfarer - Revenant"]
        self.random.shuffle(roster)
        start_count = min(int(self.options.starting_nightfarers), len(roster))
        for name in roster[:start_count]:
            self.push_precollected(self.create_item(name))
