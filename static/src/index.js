$(document).ready(function(){
  $("#vpn_italy").click(
  
    function connect(){
      $.ajax({
        url: "http://localhost:8080/connect",
        data: {
          location: "italy"
        },
        success: function( result ) {
          $("#vpn_italy_img").addClass("yellow_background");
        },
        error: function() {
          $("#vpn_italy_img").addClass("red_background");
        }
      });
    }

  );
});

// function poll(){
//   $.ajax({
//     url: "http://localhost:8080/is_connected",
//     data: {
//       location: "italy"
//     },
//     success: function( result ) {
//       $("#vpn_italy_img").addClass("yellow_background");
//     },
//     error: function() {
//       $("#vpn_italy_img").addClass("red_background");
//     }
//   });
// }
// setInterval(poll, 1000);

