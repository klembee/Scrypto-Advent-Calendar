<template>
  <div id="app">
    <h1>Scrypto Advent Calendar</h1>
    <p>25 days of scrypto examples, starting December 1st !</p>
    <div class="calendar row">
      <div class="day col-md-3 col-6" v-for="day in 25" :key="day">
        <div class="card px-2">
          <div class="card-body">
            <h5 class="card-title">Day {{ day }}</h5>
            <p class="card-text">Some quick example text to build on the card title and make up the bulk of the card's content.</p>
            <button class="btn btn-primary" :disabled="!can_be_opened(day)">
              Open
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
var dayjs = require('dayjs')

export default {
  name: 'App',
  data: function() {
    return {
      now: dayjs()
    }
  },
  methods: {
    fetch_example_files(day) {

    },
    fetch_example_metadata(day) {
      
    },
    can_be_opened: function(day) {
      return this.now.date() >= day;
    }
  }
}
</script>

<style>
  @import'~bootstrap/dist/css/bootstrap.css';
</style>
